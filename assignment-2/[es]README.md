# Trabajo Práctico 2

## Introducción

Continuamos con los desarrollos para <em>CoffeeGPT</em>. 

Coffeewards, es un sistema de puntos para fidelización de los clientes. 
Por cada compra que realizan los clientes, suman puntos que luego pueden canjear por cafes gratuitos.

## Objetivo

Deberán implementar un conjunto de aplicaciones en Rust que modele el sistema de puntos. 

## Requerimientos

- Una aplicación modelará la cafetera robot, la cual agregará o retirará puntos de las tarjetas, según un archivo de pedidos simulado. Deberán correr varias instancias en simultáneo, simulando los multiples locales de la empresa y cafeteras de cada local. 
- Cada cliente suma y comparte sus puntos con su grupo familiar; por lo tanto una misma cuenta de puntos se puede estar
  utilizando en varios locales/cafeteras en simultáneo. Se debe mantener el saldo de cada cuenta consistente.
- En el caso de pagar un café con puntos, los mismos se bloquean, pero no se descuentan hasta que el café fue efectivamente entregado al cliente. 
- La cafetera puede fallar en preparar la bebida con cierta probabilidad, debiendo devolver los puntos.
- El sistema es distribuido, cada local de CoffeeGPT tiene un servidor que mantiene los estados de cuenta. Las cafeteras se conectan con su servidor local.
- Debido a que se encuentran por todo el pais, en algunos casos con muy mala conexión, los servidores pueden entrar y salir de la red espontáneamente.
  - Mientras se encuentran fuera de red, los servidores pueden seguir acumulando puntos en las cuentas. No así retirar.
  - Al volver a conectarse, deben sincronizar los estados de cuenta


## Requerimientos no funcionales

Los siguientes son los requerimientos no funcionales para la resolución de los ejercicios:

- El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.
- Alguna de las aplicaciones implementadas debe funcionar utilizando el **modelo de actores**.
- No se permite utilizar **crates** externos, salvo los explícitamente mencionados en este enunciado, o autorizados expresamente por los profesores.
- El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.
- El código deberá funcionar en ambiente Unix / Linux.
- El programa deberá ejecutarse en la línea de comandos.
- La compilación no debe arrojar **warnings** del compilador, ni del linter **clippy**.
- Las funciones y los tipos de datos (**struct**) deben estar documentadas siguiendo el estándar de **cargo doc**.
- El código debe formatearse utilizando **cargo fmt**.
- Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

## Entrega


La resolución del presente proyecto es en grupos de tres integrantes.

La entrega del proyecto se realizará mediante Github Classroom. Cada grupo tendrá un repositorio disponible para 
hacer diferentes commits con el objetivo de resolver el problema propuesto. Se recomienda iniciar tempranamente y
hacer commits pequeños agreguen funcionalidad incrementalmente.
Se podrán hacer commits hasta el día de la entrega a las 19 hs Arg, luego el sistema automáticamente quitará el acceso
de escritura.

Asi mismo el proyecto debe incluir un informe en formato Markdown en el README.md del repositorio que contenga una 
explicación del diseño y de las decisiones tomadas para la implementación de la solución, asi como diagramas de threads y procesos,
y la comunicación entre los mismos; y diagramas de las entidades principales.

## Evaluación

### Principios teóricos y corrección de bugs

Los alumnos presentarán el código de su solución presencialmente, con foco en el uso de las diferentes herramientas de concurrencia. 
Deberán poder explicar desde los conceptos teóricos vistos en clase cómo se comportará potencialmente su solución ante 
problemas de concurrencia (por ejemplo ausencia de deadlocks).

En caso de que la solución no se comportara de forma esperada, deberán poder explicar las causas y sus posibles rectificaciones.

### Casos de prueba

Se someterá a la aplicación a diferentes casos de prueba que validen la correcta aplicación de las herramientas de concurrencia.

### Informe

El informe debe estar estructurado profesionalmente y debe poder dar cuenta de las decisiones tomadas para implementar la solución.

### Organización del código

El código debe organizarse respetando los criterios de buen diseño y en particular aprovechando las herramientas recomendadas por Rust. 
Se prohibe el uso de bloques `unsafe`. 

### Tests automatizados

La presencia de tests automatizados que prueben diferentes casos, en especial sobre el uso de las herramientas de concurrencia es un plus.

### Presentación en término

El trabajo deberá entregarse para la fecha estipulada. La presentación fuera de término sin coordinación con antelación 
con el profesor influye negativamente en la nota final.