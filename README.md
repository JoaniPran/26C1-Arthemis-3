# Taller de Programacion Arthemis-3

## Integrantes

* **Integrante 1** - [Luis Laos Pinto](https://github.com/Gonza2508)
* **Integrante 2** - [Joani Alejandro Pranteda](https://github.com/JoaniPran)
* **Integrante 3** - [Valentina Dı́az Racioppi ](https://github.com/Valudiaz78)
* **Integrante 4** - [Luis Trebejo](https://github.com/luistrebejoIt)

# Como usar

A continuación se detallan los pasos para compilar y ejecutar el programa.

## Certificado

En la máquina que va a correr el rol de Coordinador, **antes** de `docker compose
up` o `cargo run -p coordinator`, generar el cert en el host (no dentro del contenedor — el volumen lo monta
adentro solo, no hace falta correr nada de OpenSSL en la imagen):

```bash
cd DARG
mkdir -p coordinator/certs
openssl req -x509 -newkey rsa:4096 \
  -keyout coordinator/certs/coordinator.key \
  -out coordinator/certs/coordinator.pem \
  -days 365 -nodes \
  -subj "/CN=coordinator"

```

## Compilación

- Para compilar el proyecto (modo debug):

    ```bash
    cargo build
    ```

- Para compilar en modo release:

    ```bash
    cargo build --release
    ```

# Cómo correr

- El comando para ejecutar la interfaz gráfica (UI) es:

    ```bash
    cargo run -p ui <ip_coordinador>
    ```

Nota: si no es asignada una ip, se optara automaticamente por conectarse 127.0.0.1 (Localhost).

-   Sin la UI, ejecutar el coordinador con:

    ```bash
    cargo run -p coordinator
    ```

Nota: el coordinador siempre escucha en todas las interfaces (0.0.0.0) por defecto.

-   Los workers se inician así (por defecto):

    ```bash
    cargo run -p worker <nombre_a_asignar>

    cargo run -p worker <nombre_a_asignar> <ip_a_conectarse>
    ```

## Docker-Compose

- para crear una imagen y levantar el cordinador con un solo worker

    ```bash
    docker compose up --build -d
    ```

-   Para levantar el Coordinador + N Workers (ejemplo: 3 workers):

    ```bash
    docker compose up --build -d --scale worker=3
    ```

-   si unicamente se desea correr el coordinador en un contendor aparte

    ```bash
    docker compose up -d coordinator
    ```

-   Ver los logs únicamente del coordinador:

    ```bash
    docker compose logs -f coordinator
    ```

- Reconstruir e iniciar solo el coordinador:
    ```bash
    docker compose up --build -d coordinator
    ```

-   Para detener todo (conservando la base de datos):

    ```bash
    docker compose stop
    ```

-   Para destruir los contenedores y limpiar volúmenes:

    ```bash
    docker compose down -v
    ```

### Ver los logs de un worker ESPECÍFICO:

- Primero, consultá los nombres de los contenedores activos:

    ```bash
    docker compose ps
    ```

- Luego, pedí los logs de la instancia específica:

     ```bash
    docker logs -f <nombre_del_contenedor>
    ```


## Correr Wokers 

-   Para crear una imagen nueva en Docker del worker:

    ```bash
    docker build -t mi-worker .
    ```
Nota: la imagen del Docker se llamara mi-worker.

-   Para ejecutar un worker en Docker (con la red del host):

    ```bash
    docker run -d --network host --name worker_uno mi-worker sh -c "./target/release/worker <nombre_del_worker> <ip_a_conectarse>"
    ```

Nota: la opción `--name` especifica el nombre del contenedor.

El coordinador escucha en `0.0.0.0:8080`, por lo que desde Docker debes apuntar a la IP del host si `127.0.0.1` no funciona en tu entorno. Por ejemplo:

```bash
docker run -d --network host --name worker_uno mi-worker sh -c "./target/release/worker worker_docker 192.168.1.100"
```

Si estás usando `--network host` en Linux, `127.0.0.1` debería funcionar, pero en algunos entornos Docker el contenedor no puede ver al host por ese loopback. Usa la IP real de la máquina donde corre el coordinador.

- Para ver los logs del contenedor:
    ```bash
    docker logs <nombre_del_contenedor>
    ```

-   Para detener la ejecución del contenedor:

    ```bash
    docker stop <nombre_del_contenedor>
    ```

-  Para eliminar el contenedor:

    ```bash
    docker rm <nombre_del_contenedor>
    ```

- Para eliminar la imagen 

    ```bash
    docker rmi <id_o_nombre>
    ```

### Cómo testear

- Ejecuta los tests del repo con:

    ```bash
    cargo test
    ```


