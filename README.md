# Taller de Programacion Arthemis-3

## Integrantes

## Como usar

A continuación se detallan los pasos para compilar y ejecutar el programa.

### Compilación

Para compilar el proyecto (modo debug):

```bash
cargo build
```

Para compilar en modo release:

```bash
cargo build --release
```

### Cómo correr

El comando para ejecutar la interfaz gráfica (UI) es:

```bash
cargo run -p ui
```

Sin la UI, ejecutar el coordinador con:

```bash
cargo run -p coordinator
```

Nota: el coordinador siempre escucha en todas las interfaces (0.0.0.0) por defecto.

Los workers se inician así (por defecto):

```bash
cargo run -p worker <nombre_a_asignar>

cargo run -p worker <nombre_a_asignar> <ip_a_conectarse>
```

### Docker: workers

Para crear una imagen Docker del worker:

```bash
docker build -t mi-worker .
```
Nota: la imagen del Docker se llamara mi-worker.

Para ejecutar un worker en Docker (con la red del host):

```bash
docker run -d --network host --name worker_uno mi-worker sh -c "./target/release/worker <nombre_del_worker> <ip_a_conectarse>"
```

Nota: la opción `--name` especifica el nombre del contenedor.

El coordinador escucha en `0.0.0.0:8080`, por lo que desde Docker debes apuntar a la IP del host si `127.0.0.1` no funciona en tu entorno. Por ejemplo:

```bash
docker run -d --network host --name worker_uno mi-worker sh -c "./target/release/worker worker_docker 192.168.1.100"
```

Si estás usando `--network host` en Linux, `127.0.0.1` debería funcionar, pero en algunos entornos Docker el contenedor no puede ver al host por ese loopback. Usa la IP real de la máquina donde corre el coordinador.

Para ver los logs del contenedor:

```bash
docker logs <nombre_del_contenedor>
```

Para detener la ejecución del contenedor:

```bash
docker stop <nombre_del_contenedor>
```

Para eliminar el contenedor:

```bash
docker rm <nombre_del_contenedor>
```

Para eliminar la imagen 

```bash
docker rmi <id_o_nombre>
```

### Cómo testear

Ejecuta los tests del repo con:

```bash
cargo test
```

