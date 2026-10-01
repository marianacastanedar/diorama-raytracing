# Diorama Marino - Raytracing

Una isla flotante sobre el mar: casa de dos pisos con techo de tejas,
balcón, chimenea con humo animado, árbol y arbustos. Renderizada con un
raytracer por CPU escrito desde cero en Rust, con sombras, reflexión y
refracción reales en agua y vidrio, ciclo día/noche, y un skybox HDR de
fondo.

## Build

    cargo build --release

## Run

    cargo run --release

## Controles

| Tecla       | Acción                                      |
|-------------|----------------------------------------------|
| Flechas     | Orbitar cámara                                |
| + / -       | Zoom (también numpad)                         |
| R           | Reset de cámara                               |
| L           | Prender / apagar las ventanas y la chimenea   |
| N           | Saltar de día a noche (o viceversa)           |
| Esc         | Salir                                         |

## Materiales

| Material | Notas                                                  |
|----------|---------------------------------------------------------|
| Agua     | Refracción (IOR 1.33) + reflexión, plano en y=0          |
| Arena    | Difuso — pasto, copas del árbol, arbustos y tejas (patrón procedural) |
| Roca     | Difuso con specular medio — isla, torre, chimenea        |
| Madera   | Puertas, balcón, tronco del árbol                        |
| Vidrio   | Refracción (IOR 1.5) + reflexión alta — ventanas          |

## Notas técnicas

- Raytracing recursivo (profundidad máx. 4) para reflexión y refracción
  simultáneas (Snell's law, con reflexión total interna), más sombras reales
  (rayo hacia la luz, no solo el ángulo de la normal).
- El render corre sin parar en un hilo de fondo (`std::thread::scope` +
  canales `mpsc`, sin dependencias externas): la ventana nunca se congela
  esperando un frame, necesario para que las animaciones (humo, día/noche)
  se vean fluidas incluso con la cámara quieta.
- Ciclo día/noche automático (~45s por vuelta): oscurece el cielo y el sol
  gradualmente; de noche aparecen estrellas y una luna procedurales (no
  vienen en el panorama HDR), ambas visibles también en el reflejo del agua.
  Tecla `N` para saltar manualmente de día a noche.
- Ventanas emisivas (tecla `L`): se encienden con un color plano sin
  depender de luz ni sombras; apagadas, se comportan como vidrio normal. La
  chimenea titila con el mismo mecanismo cuando es de noche.
- Techo con patrón de tejas alternadas calculado desde el punto de impacto
  (sin texturas ni cubos extra).
- Skybox: panorama equirectangular HDR (Kiara 1 Dawn, Poly Haven) con un
  decoder propio del formato Radiance — el build de raylib que usa este
  proyecto no incluye soporte para `.hdr`.
- Resolución: 1280x720.

## Video demo

[Pendiente: agregar link o embed]
