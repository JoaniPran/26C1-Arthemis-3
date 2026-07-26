# TLS en la API del Coordinador (Coordinador ↔ Workers ↔ UI)

## 1. Qué se pidió

El profesor pidió que la conexión usada para transferir archivos entre el Coordinador y
los Workers pase de **HTTP** a **HTTPS**, cifrando el contenido con TLS.

## 2. Por qué se tocó solo el puerto :8081

Arthemis tiene tres canales de red distintos:

| Puerto | Protocolo | Qué transporta |
|---|---|---|
| `:8080` | Protocolo propio (JSON por línea sobre TCP crudo) | `RegisterWorker`, `Heartbeat`, `TaskStatus`, `LogFragment`, `AssignTask` (Coordinador ↔ Workers) |
| `:8081` | HTTP (servido con el crate `rouille`) | Login/registro de usuarios, workflows, tareas, subida/descarga de artefactos `.zip` |
| `:8082` | TCP propio (`event_server.rs`) | Push de eventos (`LOADED`, `ERROR`, `STATUS`, `LOG`) desde el Coordinador hacia la UI |

El pedido fue literalmente "que pase de http a https". Los puertos `:8080` y `:8082`
**nunca fueron HTTP** — son protocolos propios a medida sobre TCP crudo, así que no
tienen un equivalente "https". El único tramo que es HTTP de verdad es el `:8081`, y por
eso el cambio se limitó a ese servidor.

Esto tomó más relevancia de la que parecía en un principio: el proyecto evolucionó y
`:8081` dejó de ser solo el servidor de artefactos — ahora es la API completa de la
aplicación (login, registro, workflows, tareas, artefactos), usada tanto por los
Workers (`curl`, para bajar/subir `.zip`) como por la UI (`reqwest`, para todo lo
demás, incluyendo **usuario y contraseña en `/login` y `/register`**). Cifrar este
puerto protege bastante más que solo los archivos.

Los puertos `:8080` (comandos, logs, heartbeats) y `:8082` (eventos de UI) siguen en
texto plano. Cifrarlos también es un trabajo aparte y bastante más grande: implica
reescribir cómo se comparten los sockets entre threads en `handler.rs`, `state.rs`,
`client.rs`, `heartbeat.rs` y `executor.rs`, porque hoy dependen de
`TcpStream::try_clone()`, algo que un stream TLS no soporta de la misma manera.

## 3. Qué se necesitó

- **Un certificado y una clave privada** (autofirmados, porque es un sistema cerrado
  sin autoridad certificadora pública).
- **El feature `rustls` del crate `rouille`** (lado Coordinador), apagado por defecto
  en `Cargo.toml`. `rouille` usa por debajo a `tiny_http`, y este último expone soporte
  TLS vía dos backends opcionales: `ssl-openssl` (OpenSSL) o `ssl-rustls` (rustls, en
  Rust puro). Activar el feature `rustls` de `rouille` prende en cascada el
  `ssl-rustls` de `tiny_http`. Nunca se llama a la API de `rustls` directamente:
  `rouille` la esconde por completo detrás de `Server::new_ssl(addr, handler,
  certificate_bytes, private_key_bytes)`.
- **El feature `rustls-tls` del crate `reqwest`** (lado UI), en reemplazo de su backend
  por defecto (`default-tls`, que usa OpenSSL del sistema vía `openssl-sys` y requiere
  tener `libssl-dev` + `pkg-config` instalados). Con `rustls-tls`, el cliente HTTP de
  la UI no depende de ninguna librería del sistema operativo — importante porque cada
  integrante del grupo compila en su propia máquina.
- **Nada nuevo del lado del Worker**: sigue shelleando `curl`, que ya trae su propio
  soporte TLS incluido en el binario del sistema operativo.

## 4. Cambios hechos, archivo por archivo

### `coordinator/Cargo.toml`
```toml
rouille = { version = "3.6", features = ["rustls"] }
```
Sin este feature, `Server::new_ssl` ni siquiera existe en el crate.

### `coordinator/src/artifact_server.rs`
- Se leen los bytes crudos de `coordinator/certs/coordinator.pem` y
  `coordinator/certs/coordinator.key` con `fs::read` (rutas relativas a donde se
  ejecuta `cargo run`, igual que ya se hacía con `./workflows` o `arthemis.db`).
- `rouille::start_server(addr, closure)` pasó a ser
  `rouille::Server::new_ssl(addr, closure, certificate, private_key).unwrap().run()`.
  La lógica de **todas** las rutas (`/login`, `/register`, `/workflows/`, `/tasks/`,
  `/start_task`, `/reset_workflow`, `/upload_workflow/`, `/workers_count`,
  `/download/`, `/upload/`) **no cambió ni una línea**: TLS es un problema de la capa
  de transporte, no de la lógica HTTP. El parámetro `database: Arc<Mutex<Database>>`
  que ya tenía la función se preservó sin tocar.

### `worker/src/executor.rs`
- Los dos comandos `curl` (descarga y subida de artefactos) pasaron de `http://` a
  `https://`.
- Se agregó el flag **`-k`** (`--insecure`) a ambos.

### `ui/Cargo.toml`
```toml
reqwest = { version = "0.11", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```
Se desactivó `default-tls` (OpenSSL del sistema) y se activó `rustls-tls`.

### `ui/src/utils.rs`
Se agregó un helper compartido, para no repetir la misma configuración en cada uno de
los ~7 lugares que hacen una petición HTTP:
```rust
pub fn insecure_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("No se pudo construir el cliente HTTP")
}
```
`danger_accept_invalid_certs(true)` es, para `reqwest`, el equivalente exacto de `-k`
en `curl`.

### `ui/src/app.rs`, `ui/src/ui/central.rs`, `ui/src/ui/login.rs`
En cada uno de los puntos de contacto con el Coordinador:
- `http://` → `https://`.
- `reqwest::blocking::Client::new()` (o el atajo `reqwest::blocking::get(&url)`, que
  usa un cliente implícito sin forma de configurar TLS) → `crate::utils::insecure_client()`.

Puntos afectados: `/workers_count` y `/tasks/{user}/{workflow}` (`app.rs`),
`/reset_workflow` y `/start_task` (`central.rs`), `/login`, `/register` y
`/workflows/{user}` (`login.rs`), `/upload_workflow/{user}/{file}` (`utils.rs`).

### `.gitignore`
- Se agregó `DARG/coordinator/certs/` para que la clave privada y el certificado
  nunca se suban al repositorio.

## 5. Por qué no se valida el certificado (`-k` / `danger_accept_invalid_certs`)

Validar de verdad el certificado (`curl --cacert`, o un `reqwest` sin
`danger_accept_invalid_certs`) exige que la URL usada para conectarse coincida con lo
que el certificado declara como Subject Alternative Name (SAN). Como tanto el Worker
como la UI se conectan al Coordinador por una **IP que puede variar según en qué
máquina se corra la demo** (`stream.peer_addr()` en el Worker, `ui_state.coordinator_ip`
en la UI), un certificado autofirmado genérico no la va a tener registrada como SAN.
Se probó en este entorno y falla así:

```
curl: (60) SSL: certificate subject name 'coordinator' does not match target host name '127.0.0.1'
```

Por eso se optó por `-k` / `danger_accept_invalid_certs(true)`: la conexión **sigue
viajando cifrada por TLS** (nadie que intercepte el tráfico puede leer usuario,
contraseña, ni el contenido de los `.zip`), pero ni el Worker ni la UI verifican la
identidad del certificado del Coordinador. Esto deja abierta una ventana teórica a un
ataque de tipo man-in-the-middle (alguien en la misma red que se haga pasar por el
Coordinador). Para un trabajo de facultad es un trade-off razonable, pero si en algún
momento el Coordinador va a correr siempre en una IP fija conocida, conviene pasar a
validación real con un certificado que incluya esa IP como SAN (ver sección
siguiente).

## 6. Cómo generar el cert/key si otra computadora va a ser el Coordinador

Cada máquina que vaya a correr el rol de **Coordinador** necesita su propio par
cert/key en `DARG/coordinator/certs/`. No se versiona en git (está en `.gitignore`),
así que hay que generarlo a mano una vez por máquina:

```bash
cd DARG
mkdir -p coordinator/certs
openssl req -x509 -newkey rsa:4096 \
  -keyout coordinator/certs/coordinator.key \
  -out coordinator/certs/coordinator.pem \
  -days 365 -nodes \
  -subj "/CN=coordinator"
```

- `-nodes`: la clave privada queda sin passphrase (necesario porque el Coordinador la
  lee automáticamente al arrancar, sin que nadie tipee una contraseña).
- `-days 365`: validez de un año; para este proyecto se puede alargar sin problema
  (`-days 3650`, por ejemplo) ya que no hay renovación automática implementada.
- **Ni el Worker ni la UI necesitan ningún archivo de certificado** — como ambos usan
  `-k` / `danger_accept_invalid_certs`, no validan nada del lado cliente. Solo
  necesitan poder llegar por red al puerto `:8081` del Coordinador.
- Si `coordinator/certs/` no existe o los archivos no están, **el proceso completo
  no se cae** — el servidor HTTPS corre en su propio thread (`server.rs` lo lanza con
  `thread::spawn`), así que el panic de `fs::read(...).expect(...)` solo mata ese
  thread. `:8080` (Workers) y `:8082` (eventos de UI) siguen respondiendo con
  normalidad, y a simple vista parece que todo anda — hasta que alguien intenta
  loguearse, listar workflows o subir/bajar un artefacto y `:8081` no contesta nada.
  Se verificó esto en Docker (ver sección 8) y es exactamente lo que pasa: el
  contenedor queda "Up" y el worker se conecta bien, pero el login falla.

### Si en el futuro se quiere validar el certificado de verdad

1. Conocer de antemano la IP (o el hostname) fija del Coordinador.
2. Generar el cert incluyendo esa IP como SAN, por ejemplo:
   ```bash
   openssl req -x509 -newkey rsa:4096 \
     -keyout coordinator/certs/coordinator.key \
     -out coordinator/certs/coordinator.pem \
     -days 365 -nodes \
     -subj "/CN=coordinator" \
     -addext "subjectAltName=IP:<IP_DEL_COORDINADOR>,DNS:localhost,IP:127.0.0.1"
   ```
3. Copiar el `.pem` (nunca el `.key`) a cada Worker y a la máquina de cada UI.
4. Reemplazar `-k` por `curl --cacert <ruta>/coordinator.pem` en `executor.rs`, y
   `insecure_client()` por un `reqwest::blocking::Client` construido con
   `.add_root_certificate(...)` cargando ese mismo `.pem`, en vez de
   `danger_accept_invalid_certs(true)`.

## 7. Cómo verificar que anduvo

Con el Coordinador corriendo (`cargo run -p coordinator` desde `DARG/`):

```bash
# Debe responder (cifrado, sin validar el cert):
curl -k https://127.0.0.1:8081/download/algo_que_no_existe.zip
# -> "Archivo no encontrado" con HTTP 404

curl -k -X POST https://127.0.0.1:8081/register \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","password":"demo"}'
# -> {"user_id":1,"username":"demo"} con HTTP 200

curl -k https://127.0.0.1:8081/workers_count
# -> "0" (o la cantidad de workers conectados)

# Debe fallar (cert autofirmado, sin -k):
curl https://127.0.0.1:8081/workers_count
# -> curl: (60) SSL certificate problem

# Debe fallar (ya no hay HTTP plano en ese puerto):
curl http://127.0.0.1:8081/workers_count
# -> Empty reply from server
```

Y de punta a punta: levantar `cargo run -p ui`, loguearse o registrarse, subir un
workflow `.yaml`, e iniciar una tarea — todo eso ahora viaja por HTTPS hacia el
Coordinador.

## 8. Docker

El `Dockerfile` solo copia los **binarios** compilados (`coordinator`, `worker`) a la
imagen final, nunca el código fuente. Como `coordinator/certs/` está en `.gitignore`
y ni siquiera se copia dentro de la imagen, **el certificado tiene que entrar por
volumen**, igual que ya se hacía con `./workflows`. Se agregó en
`docker-compose.yml`:

```yaml
services:
  coordinator:
    volumes:
      - ./workflows:/app/workflows
      - ./coordinator/certs:/usr/src/coordinator/certs:ro
```

`/usr/src` es el `WORKDIR` real dentro del contenedor (coincide con la ruta relativa
`coordinator/certs/...` que usa `fs::read` en `artifact_server.rs`). El worker no
necesita ningún volumen nuevo — sigue usando `curl -k`, no valida ningún cert.

**Se probó en este entorno, con el daemon de Docker real** (no es teoría):

- Sin el volumen montado: `docker compose up` levanta los dos contenedores y el
  worker se conecta sin problema por `:8080` — pero el thread del servidor HTTPS
  paniquea al arrancar (`No se pudo leer el certificado TLS`) y el contenedor queda
  "Up" igual, con `:8081` completamente muerto y sin ningún reinicio ni error
  visible en `docker compose ps`. Es un fallo silencioso: hay que generar el
  certificado en el host **antes** de `docker compose up`.
- Con el volumen montado (como quedó arriba): el coordinador levanta los tres
  puertos, el worker se registra, y `curl -k https://127.0.0.1:8081/...` responde
  bien desde afuera del contenedor.

### Cómo levantarlo

En la máquina que va a correr el rol de Coordinador, **antes** de `docker compose
up`, generar el cert en el host (no dentro del contenedor — el volumen lo monta
adentro solo, no hace falta correr nada de OpenSSL en la imagen):

```bash
cd DARG
mkdir -p coordinator/certs
openssl req -x509 -newkey rsa:4096 \
  -keyout coordinator/certs/coordinator.key \
  -out coordinator/certs/coordinator.pem \
  -days 365 -nodes \
  -subj "/CN=coordinator"

docker compose up -d
```

### Un bug preexistente, sin relación con TLS, que conviene saber

El volumen `./workflows:/app/workflows` no apunta a donde la app realmente busca los
workflows: como el `WORKDIR` es `/usr/src`, el Coordinador crea y lee
`/usr/src/workflows`, no `/app/workflows`. Hoy ese volumen es, en la práctica, un
directorio que nadie usa — un `.yaml` puesto en `DARG/workflows/` en el host no lo
va a ver el Coordinador corriendo en Docker. No se corrigió como parte de este
trabajo (no tiene que ver con HTTPS), pero es la misma categoría de problema que el
certificado, así que quedó documentado acá para no perderlo de vista.
