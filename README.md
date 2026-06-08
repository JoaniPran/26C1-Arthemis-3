# Taller de Programacion {Grupo}

## Integrantes

## Como usar 

A continuacion se detallan los pasos para compilar y ejecutar el programa.

### Compilacion

Desde el directorio DARG, correr

```cargo build --workspace```


### Como correr

Iniciar el coordinator
```cargo run -p coordinator -- 8080```

Iniciar un worker
```cargo run -p worker -- worker-1 0.0.0.0:8080```

Iniciar el UI
```cargo run -p ui```

Si el `coordinator` ya esta usando el puerto `8080`, la UI puede arrancar sin coordinador embebido

```cargo run -p ui -- --no-coordinator```

Tambien se puede indicar otro puerto o direccion al coordinador

```cargo run -p ui -- --coordinator-port 8081```

o

```cargo run -p ui -- --coordinator-addr [direccion-puerto]```

### Como testear

```cargo test -p coordinator``` 