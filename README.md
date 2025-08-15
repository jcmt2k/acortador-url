# Proyecto: Acortador de URLs con Rust y Docker

¡Bienvenido! Este repositorio contiene el código fuente de un servicio para acortar URLs, similar a Bitly o TinyURL, construido enteramente en Rust. El proyecto está diseñado como una guía de aprendizaje para personas que están explorando el desarrollo backend con Rust y la contenedorización con Docker.

## ✨ Características

*   **API RESTful:** Endpoints claros para acortar URLs y redirigir a las originales.
*   **IDs Personalizados:** Permite a los usuarios elegir sus propios alias para las URLs.
*   **Prevención de Duplicados:** Ahorra espacio y mantiene la consistencia al no generar nuevos enlaces para URLs ya acortadas.
*   **Estadísticas de Clics:** Rastrea cuántas veces se ha utilizado cada enlace corto.
*   **Validación de Datos:** Se asegura de que solo se acorten URLs válidas.
*   **Manejo de Errores Robusto:** La API devuelve códigos de estado y mensajes de error claros.
*   **Configuración Flexible:** El host y el puerto se pueden configurar mediante variables de entorno.
*   **Contenerizado con Docker:** Incluye un `Dockerfile` para un despliegue fácil y reproducible.
*   **Probado:** Incluye un conjunto de pruebas de integración para garantizar la fiabilidad.

## 📋 Prerrequisitos

Antes de empezar, asegúrate de tener instaladas las siguientes herramientas:

*   **Rust:** [Instrucciones de instalación](https://www.rust-lang.org/tools/install)
*   **Docker:** [Instrucciones de instalación](https://docs.docker.com/engine/install/)
*   **`sqlx-cli`:** Para manejar las migraciones de la base de datos.
    ```bash
    cargo install sqlx-cli
    ```

## 🚀 Cómo Empezar (Localmente)

Sigue estos pasos para ejecutar la aplicación en tu máquina local.

### 1. Clona el Repositorio

```bash
git clone <URL-DEL-REPOSITORIO>
cd acortador-url
```

### 2. Configura las Variables de Entorno

Crea un archivo llamado `.env` en la raíz del proyecto. Este archivo contendrá la URL de conexión a nuestra base de datos SQLite.

```bash
echo "DATABASE_URL=sqlite:db.sqlite" > .env
```

### 3. Prepara la Base de Datos

`sqlx-cli` usará el archivo `.env` para encontrar la base de datos y aplicar las migraciones necesarias para crear las tablas.

```bash
sqlx database create
sqlx migrate run
```

### 4. Ejecuta la Aplicación

¡Ya está todo listo! Ahora puedes iniciar el servidor.

```bash
cargo run
```

Verás un mensaje que dice `listening on 127.0.0.1:3000`. Ahora puedes abrir tu navegador en [http://localhost:3000](http://localhost:3000) para ver la página de inicio.

## 🐳 Ejecutar con Docker

Gracias a Docker, puedes construir y ejecutar el proyecto en un contenedor aislado sin necesidad de tener Rust instalado en tu máquina (solo Docker).

### 1. Construye la Imagen

Este comando leerá el `Dockerfile`, descargará las dependencias necesarias y compilará la aplicación, empaquetándolo todo en una imagen llamada `acortador-url`.

```bash
sudo docker build -t acortador-url .
```

### 2. Ejecuta el Contenedor

Una vez construida la imagen, puedes iniciar un contenedor a partir de ella.

```bash
sudo docker run -p 3000:3000 --name mi-acortador -d acortador-url
```

*   `-p 3000:3000`: Mapea el puerto 3000 de tu máquina al puerto 3000 del contenedor.
*   `--name mi-acortador`: Le da un nombre fácil de recordar a tu contenedor.
*   `-d`: Ejecuta el contenedor en segundo plano.

La aplicación ahora está corriendo dentro del contenedor y es accesible en [http://localhost:3000](http://localhost:3000).

## 🛠️ Endpoints de la API

La API proporciona los siguientes endpoints:

*   `POST /shorten`: Acorta una nueva URL.
*   `GET /{id}`: Redirige a la URL original.
*   `GET /stats/{id}`: Muestra las estadísticas de un enlace.

### `POST /shorten`

Acorta una URL.

**Body (JSON):**
```json
{
  "url": "https://www.rust-lang.org/",
  "custom_id": "rust-oficial" // Opcional
}
```

**Respuesta (201 Created):**
```json
{
  "url": "http://localhost:3000/rust-oficial"
}
```

### `GET /{id}`

Redirige a la URL original con un código de estado `303 See Other`.

### `GET /stats/{id}`

Obtiene las estadísticas de un enlace corto.

**Respuesta (200 OK):**
```json
{
  "url": "https://www.rust-lang.org/",
  "clicks": 42
}
```

## 🗺️ El Viaje del Desarrollo: 9 Mejoras

Este proyecto no se construyó de una sola vez. Empezó como un simple "Hola, Mundo" con `axum` y evolucionó a través de 9 mejoras clave. A continuación, detallamos cada paso para que puedas entender el "porqué" de cada decisión.

### Mejora 1: Validación de URLs

*   **Objetivo:** Asegurarnos de que los usuarios solo puedan acortar URLs válidas.
*   **Implementación:** Añadimos la librería `validator`. En el `struct` `ShortenRequest`, usamos el atributo `#[validate(url)]` en el campo `url`.
*   **Por qué:** Sin validación, un usuario podría enviar cualquier texto ("hola mundo"), y lo guardaríamos como si fuera un enlace. Esto corrompe la lógica de nuestra aplicación. La validación en la entrada es una de las primeras líneas de defensa para un servicio robusto.

### Mejora 2: Gestión de Errores Robusta

*   **Objetivo:** Eliminar los `.unwrap()` que pueden causar que la aplicación "entre en pánico" (cierre inesperado) y, en su lugar, devolver respuestas de error HTTP claras.
*   **Implementación:** Creamos un `enum AppError` personalizado. Implementamos la conversión (`From`) de los errores de otras librerías (como `sqlx::Error`) a nuestro `AppError`. Finalmente, implementamos `IntoResponse` para `AppError`, lo que nos permite mapear cada tipo de error a un código de estado HTTP específico (ej. `404 Not Found`, `500 Internal Server Error`).
*   **Por qué:** Un servidor nunca debe "morir". Si la base de datos no está disponible, la API debe seguir en línea y responder con un error 500, en lugar de cerrarse. Esto hace que el servicio sea mucho más fiable.

### Mejora 3: Configuración de Host y Puerto

*   **Objetivo:** Permitir que el host y el puerto del servidor se configuren desde fuera del código.
*   **Implementación:** Usamos `std::env::var` para leer las variables de entorno `HOST` y `PORT` al iniciar la aplicación, con valores por defecto (`127.0.0.1` y `3000`) si no se proporcionan.
*   **Por qué:** Esto es fundamental para el despliegue. En un entorno de producción, es muy probable que necesitemos que el servidor escuche en `0.0.0.0` en lugar de `127.0.0.1` para ser accesible desde fuera del contenedor o la máquina.

### Mejora 4: Página de Inicio Simple

*   **Objetivo:** Ofrecer una interfaz de usuario web básica para interactuar con el servicio.
*   **Implementación:** Creamos una nueva ruta (`/`) que sirve un archivo HTML estático. Este HTML contiene un formulario y un poco de JavaScript para enviar la petición a nuestro endpoint `/shorten` y mostrar el resultado dinámicamente.
*   **Por qué:** Una API es genial para desarrolladores, pero una interfaz gráfica, por simple que sea, hace que la herramienta sea accesible para todo el mundo.

### Mejora 5: IDs Personalizados

*   **Objetivo:** Dar a los usuarios la opción de elegir su propio alias para un enlace corto.
*   **Implementación:** Añadimos un campo opcional `custom_id: Option<String>` al `struct` `ShortenRequest`. En la lógica del handler, si este campo existe, lo usamos como ID. También añadimos una comprobación para asegurar que el ID personalizado no esté ya en uso, devolviendo un error `409 Conflict` si lo está.
*   **Por qué:** Esto añade una característica de personalización muy demandada en este tipo de servicios, mejorando la experiencia del usuario.

### Mejora 6: Evitar Duplicados

*   **Objetivo:** Si un usuario intenta acortar una URL que ya existe, devolver el enlace corto existente en lugar de crear uno nuevo.
*   **Implementación:** Antes de insertar una nueva URL, realizamos una consulta `SELECT` para ver si la `original_url` ya está en la base de datos. Si la encontramos, devolvemos el `id` existente con un código de estado `200 OK` en lugar de `201 Created`.
*   **Por qué:** Esto evita la duplicación de datos, mantiene la base de datos más limpia y asegura que una misma URL siempre tenga el mismo enlace corto, lo cual es un comportamiento predecible y deseable.

### Mejora 7: Estadísticas de Clics

*   **Objetivo:** Rastrear y mostrar cuántas veces se ha visitado un enlace.
*   **Implementación:** Añadimos una columna `clicks` a nuestra tabla `urls` con una nueva migración. En el handler `redirect`, ejecutamos una consulta `UPDATE` para incrementar este contador cada vez que se visita un enlace. Creamos un nuevo endpoint `/stats/{id}` que devuelve la URL original y el contador de clics.
*   **Por qué:** Las estadísticas son una característica de gran valor añadido. Permiten a los usuarios medir el impacto de sus enlaces.

### Mejora 8: Pruebas Unitarias y de Integración

*   **Objetivo:** Crear una red de seguridad que verifique que toda nuestra lógica funciona como se espera y que nos avise si futuros cambios rompen algo.
*   **Implementación:** Creamos un archivo `tests/api.rs`. Usando `tokio::test` y librerías como `tower::ServiceExt`, escribimos pruebas que simulan peticiones HTTP a nuestra API y verifican que las respuestas (códigos de estado, cuerpos JSON, cabeceras) sean las correctas para diferentes escenarios (casos de éxito, errores, casos límite).
*   **Por qué:** El software sin pruebas es software roto esperando a suceder. Las pruebas automatizadas son la única forma de escalar un proyecto y modificarlo con la confianza de que no estamos introduciendo errores.

### Mejora 9: Contenerización con Docker

*   **Objetivo:** Empaquetar nuestra aplicación y todas sus dependencias en una imagen portátil, auto-contenida y reproducible que se pueda ejecutar en cualquier máquina que tenga Docker.
*   **Por qué:** Esto resuelve el clásico problema de "en mi máquina funciona". Docker asegura que el entorno de desarrollo, pruebas y producción sea idéntico, eliminando una enorme fuente de problemas y simplificando radicalmente el despliegue.

#### El `Dockerfile`: Nuestro Plano de Construcción

Un `Dockerfile` es un archivo de texto que contiene una serie de instrucciones sobre cómo construir una imagen de Docker. La nuestra utiliza una técnica llamada **compilación multi-etapa**, que es una práctica recomendada para crear imágenes pequeñas y seguras.

**Etapa 1: El Constructor (`builder`)**

```dockerfile
FROM rust:1.89 as builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y libsqlite3-dev
RUN cargo install sqlx-cli

COPY . .

RUN cargo build --release
```

1.  `FROM rust:1.89 as builder`: Empezamos con una imagen oficial de Rust que contiene todas las herramientas necesarias para compilar nuestro código (el compilador `rustc`, `cargo`, etc.). La nombramos `builder`.
2.  `WORKDIR /usr/src/app`: Creamos un directorio de trabajo dentro de la imagen para mantener todo organizado.
3.  `RUN ...`: Ejecutamos comandos para instalar `libsqlite3-dev` (una dependencia del sistema necesaria para compilar `sqlx`) y `sqlx-cli`.
4.  `COPY . .`: Copiamos todo nuestro código fuente (el `.` local) al directorio de trabajo (`.` en la imagen).
5.  `RUN cargo build --release`: Este es el paso clave. `cargo` compila nuestro proyecto con optimizaciones (`--release`), creando un único archivo binario ejecutable.

Al final de esta etapa, tenemos una imagen grande que contiene el código fuente, todas las herramientas de compilación y nuestro binario final. Pero no necesitamos todo eso para *ejecutar* la aplicación.

**Etapa 2: La Imagen Final**

```dockerfile
FROM debian:stable-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/acortador-url .
COPY --from=builder /usr/local/cargo/bin/sqlx .

COPY templates ./templates
COPY migrations ./migrations
COPY entrypoint.sh .

EXPOSE 3000

ENV DATABASE_URL=sqlite:db.sqlite
ENV HOST=0.0.0.0
ENV PORT=3000

ENTRYPOINT ["./entrypoint.sh"]
```

1.  `FROM debian:stable-slim`: Empezamos de nuevo, pero esta vez con una imagen de Debian muy ligera. No tiene Rust ni ninguna herramienta de compilación, lo que la hace mucho más pequeña y segura.
2.  `COPY --from=builder ...`: Aquí está la magia de la compilación multi-etapa. Copiamos selectivamente solo los artefactos que necesitamos de la etapa `builder`: nuestro binario (`acortador-url`) y el de `sqlx-cli`.
3.  `COPY ...`: También copiamos los archivos necesarios para la ejecución: las plantillas HTML, las migraciones de la base de datos y nuestro script de entrada.
4.  `EXPOSE 3000`: Informamos a Docker que el contenedor escuchará en el puerto 3000.
5.  `ENV ...`: Configuramos las variables de entorno que nuestra aplicación necesita para ejecutarse dentro del contenedor.
6.  `ENTRYPOINT ["./entrypoint.sh"]`: Este es el comando que se ejecutará cuando se inicie el contenedor.

#### El `entrypoint.sh`: Preparando el Terreno

Este script se asegura de que la base de datos esté lista *antes* de que se inicie el servidor principal.

```sh
#!/bin/sh

# Crea el archivo de la base de datos si no existe
touch db.sqlite

# Configura la base de datos y ejecuta las migraciones
sqlx database setup

# Inicia la aplicación
./acortador-url
```

Este enfoque hace que nuestro contenedor sea completamente auto-suficiente. Cualquiera puede ejecutarlo y la base de datos se creará y configurará automáticamente al primer inicio.

---

¡Gracias por explorar este proyecto! Esperamos que te haya sido útil para aprender y experimentar con Rust y Docker.
