# TLS en la transferencia de artefactos (Coordinador ↔ Workers)

## 1. Qué se pidió

El profesor pidió que la conexión usada para transferir archivos entre el Coordinador y
los Workers pase de **HTTP** a **HTTPS**, cifrando el contenido con TLS.

## 2. Por qué se tocó solo el puerto :8081 y no el :8080

Arthemis tiene dos canales de red distintos:

| Puerto | Protocolo | Qué transporta |
|---|---|---|
| `:8080` | Protocolo propio (JSON por línea sobre TCP crudo) | `RegisterWorker`, `Heartbeat`, `TaskStatus`, `LogFragment`, `AssignTask` |
| `:8081` | HTTP (servido con el crate `rouille`) | Subida/descarga de los `.zip` de artefactos (`produces`/`consumes`) |

El pedido fue literalmente "que pase de http a https". El `:8080` **nunca fue HTTP** —
es un protocolo propio a medida sobre TCP— así que no tiene un equivalente "https". El
único tramo que es HTTP de verdad, y el que efectivamente mueve los archivos, es el
`:8081`. Por eso el cambio se limitó a ese servidor. El `:8080` (comandos, logs,
heartbeats) sigue en texto plano; si en algún momento se pide cifrar también eso, es un
trabajo aparte y bastante más grande (implica reescribir cómo se comparten los sockets
entre threads en `handler.rs`, `state.rs`, `client.rs`, `heartbeat.rs` y `executor.rs`,
porque hoy dependen de `TcpStream::try_clone()`, algo que un stream TLS no soporta de la
misma manera).

## 3. Qué se necesitó

- **Un certificado y una clave privada** (autofirmados, porque es un sistema cerrado sin
  autoridad certificadora pública).
- **El feature `rustls` del crate `rouille`**, que en `Cargo.toml` está apagado por
  defecto. `rouille` usa por debajo a `tiny_http`, y este último expone soporte TLS vía
  dos backends opcionales: `ssl-openssl` (OpenSSL) o `ssl-rustls` (rustls, en Rust puro).
  Activar el feature `rustls` de `rouille` prende en cascada el `ssl-rustls` de
  `tiny_http`. Nunca se llama a la API de `rustls` directamente: `rouille` la esconde
  por completo detrás de `Server::new_ssl(addr, handler, certificate_bytes,
  private_key_bytes)`.

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
  La lógica de las rutas `/download/` y `/upload/` **no cambió ni una línea**: TLS es un
  problema de la capa de transporte, no de la lógica HTTP.

### `worker/src/executor.rs`
- Los dos comandos `curl` (descarga y subida de artefactos) pasaron de `http://` a
  `https://`.
- Se agregó el flag **`-k`** (`--insecure`) a ambos.

### `.gitignore`
- Se agregó `DARG/coordinator/certs/` para que la clave privada y el certificado nunca
  se suban al repositorio.

## 5. Por qué `-k` y no `--cacert`

`curl --cacert archivo.pem` valida dos cosas: que el certificado esté firmado por ese
archivo, **y** que el hostname/IP de la URL coincida con el certificado (Subject
Alternative Name). Como el worker se conecta usando la IP real del coordinador
(`stream.peer_addr()`), y esa IP puede cambiar según en qué máquina se corra la demo, un
certificado autofirmado genérico no la va a tener registrada como SAN. Se probó en este
entorno y falla así:

```
curl: (60) SSL: certificate subject name 'coordinator' does not match target host name '127.0.0.1'
```

Por eso se usa `-k`: la conexión **sigue viajando cifrada por TLS** (nadie que
intercepte el tráfico puede leer el contenido de los `.zip`), pero el worker no verifica
la identidad del certificado del coordinador. Esto deja abierta una ventana teórica a un
ataque de tipo man-in-the-middle (alguien en la misma red que se haga pasar por el
coordinador). Para un trabajo de facultad es un trade-off razonable, pero si en algún
momento el coordinador va a correr siempre en una IP fija conocida, conviene volver a
`--cacert` con un certificado que incluya esa IP como SAN (ver sección siguiente).

## 6. Cómo generar el cert/key si otra computadora va a ser el Coordinador

Cada máquina que vaya a correr el rol de **Coordinador** necesita su propio par
cert/key en `DARG/coordinator/certs/`. No se versiona en git (está en `.gitignore`), así
que hay que generarlo a mano una vez por máquina:

```bash
cd DARG
mkdir -p coordinator/certs
openssl req -x509 -newkey rsa:4096 \
  -keyout coordinator/certs/coordinator.key \
  -out coordinator/certs/coordinator.pem \
  -days 365 -nodes \
  -subj "/CN=coordinator"
```

- `-nodes`: la clave privada queda sin passphrase (necesario porque el coordinador la
  lee automáticamente al arrancar, sin que nadie tipee una contraseña).
- `-days 365`: validez de un año; para este proyecto se puede alargar sin problema
  (`-days 3650`, por ejemplo) ya que no hay renovación automática implementada.
- Los workers **no necesitan ningún archivo de certificado** — como se usa `-k`, no
  validan nada del lado cliente. Solo necesitan poder llegar por red al puerto `:8081`
  del coordinador.

Si en el futuro se decide pasar a `--cacert` (validación real del certificado), ahí sí
habría que:
1. Conocer de antemano la IP (o el hostname) fija del coordinador.
2. Generar el cert incluyendo esa IP como SAN, por ejemplo:
   ```bash
   openssl req -x509 -newkey rsa:4096 \
     -keyout coordinator/certs/coordinator.key \
     -out coordinator/certs/coordinator.pem \
     -days 365 -nodes \
     -subj "/CN=coordinator" \
     -addext "subjectAltName=IP:<IP_DEL_COORDINADOR>,DNS:localhost,IP:127.0.0.1"
   ```
3. Copiar el `.pem` (nunca el `.key`) a cada worker y usar `curl --cacert
   <ruta>/coordinator.pem` en vez de `-k` en `executor.rs`.

## 7. Cómo verificar que anduvo

Con el coordinador corriendo (`cargo run -p coordinator` desde `DARG/`):

```bash
# Debe responder (cifrado, sin validar el cert):
curl -k https://127.0.0.1:8081/download/algo_que_no_existe.zip
# -> "Archivo no encontrado" con HTTP 404

# Debe fallar (cert autofirmado, sin -k):
curl https://127.0.0.1:8081/download/algo_que_no_existe.zip
# -> curl: (60) SSL certificate problem

# Debe fallar (ya no hay HTTP plano en ese puerto):
curl http://127.0.0.1:8081/download/algo_que_no_existe.zip
# -> Empty reply from server
```
