# Diorama Marino - Raytracing

Proyecto 2 de gráficas por computadora: un diorama marino minimalista
(isla de arena, acantilado de roca, muelle de madera y boyas de vidrio)
renderizado con un raytracer por CPU escrito desde cero en Rust, con
refracción y reflexión reales en agua y vidrio, y un skybox HDR de fondo.

## Build

    cargo build --release

## Run

    cargo run --release

## Controles

| Tecla       | Acción                    |
|-------------|---------------------------|
| Flechas     | Orbitar cámara            |
| + / -       | Zoom (también numpad)     |
| R           | Reset de cámara           |
| Esc         | Salir                     |

## Materiales

| Material | Notas                                          |
|----------|-------------------------------------------------|
| Agua     | Refracción (IOR 1.33) + reflexión, plano principal |
| Arena    | Difuso, base de la isla                          |
| Roca     | Difuso con specular medio, acantilado            |
| Madera   | Muelle y estructuras                             |
| Vidrio   | Refracción (IOR 1.5) + reflexión alta, boyas      |

## Notas técnicas

- Raytracing recursivo (profundidad máx. 4) para reflexión y refracción
  simultáneas (Snell's law, con reflexión total interna).
- Render paralelizado por filas con `std::thread::scope` (sin dependencias
  externas): baja de ~200ms a ~50ms por frame en un i7.
- Antialiasing progresivo: 1 rayo/pixel mientras se mueve la cámara (para
  que el giro se sienta fluido), supersampling 2x2 en cuanto se suelta.
- Skybox: panorama equirectangular HDR (Kiara 1 Dawn, Poly Haven) con un
  decoder propio del formato Radiance — el build de raylib que usa este
  proyecto no incluye soporte para `.hdr`.
- Resolución: 1280x720.

## Video demo

[Pendiente: agregar link o embed]
