# Trabajo Práctico 1

## Introducción

<em>CoffeeGPT</em> es una nueva cadena de cafeterías que pretende competir con tantas otras con un novedoso 
modelo de negocio: los clientes son servidos por robots que preparan sus órdenes de forma totalmente automática.

De entre los multiples desarrollos necesarios para implementar este proyecto, nos han contratado para realizar el software
que controlará las cafeteras espresso.

## Objetivo

Deberán implementar una aplicación en Rust que modele el sistema de control y reporte de la cafetera inteligente. 

Cada pedido de bebida a preparar se leerá como una línea de un archivo y los movimientos de los distintos actuadores se simularán con un sleep. 

## Requerimientos

- Cada cafetera dispone de N dispensadores independientes que pueden aplicar agua caliente, café molido, cacao o espuma de leche.

- Además la cafetera esta compuesta por:
  - Un contenedor de granos para moler de capacidad G
  - Un contenedor para granos molidos de capacidad M
  - Un contenedor de leche fría de capacidad L
  - Un contenedor de espuma de leche de capacidad E
  - Un contenedor de cacao de capacidad C
  - Un contenedor de agua, donde esta se toma de la red y luego calienta, de capacidad A.

- Cada pedido se representa con una línea en un archivo de texto que indicará las cantidades de café molido, agua caliente, cacao y/o espuma de leche necesarias para preparar la bebida solicitada.

- Las cantidades se "aplicarán" como tiempos de sleep además de descontar del correspondiente contenedor.

- Un solo dispenser a por vez puede tomar ingredientes de cada contenedor, es decir, no es posible por ejemplo que dos disponsers tomen café concurrentemente.

- Cuando el contenedor de cafe molido se agota, el molinillo automático toma una cantidad de granos y los convierte en café molido. Este proceso toma tiempo y no se puede obtener café durante el mismo.
 
- Análogamente sucede lo mismo con la leche y la espuma, y con el agua caliente. 

- El sistema debe minimizar los tiempos de espera de los clientes, maximizando la producción concurrente. 

- El sistema debe alertar por consola cuando los contenedores de granos, leche y cacao se encuentran por debajo de X% de capacidad.

- El sistema debe presentar periódicamente estadísticas con los niveles de todos los contenedores, cantidad total de bebidas preparadas y cantidad total de ingredientes consumidos.


## Requerimientos no funcionales

Los siguientes son los requerimientos no funcionales para la resolución de los ejercicios:

- El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.
- Se deberán utilizar las herramientas de concurrencia correspondientes al modelo de estado mutable compartido.
- No se permite utilizar **crates** externos, salvo los explícitamente mencionados en este enunciado, o autorizados expresamente por los profesores.
- El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.
- El código deberá funcionar en ambiente Unix / Linux.
- El programa deberá ejecutarse en la línea de comandos.
- La compilación no debe arrojar **warnings** del compilador, ni del linter **clippy**.
- Las funciones y los tipos de datos (**struct**) deben estar documentadas siguiendo el estándar de **cargo doc**.
- El código debe formatearse utilizando **cargo fmt**.
- Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

## Entrega

La resolución del presente proyecto es individual.

La entrega del proyecto se realizará mediante Github Classroom. Cada estudiante tendrá un repositorio disponible para 
ir haciendo diferentes commits con el objetivo de resolver el problema propuesto. Se recomienda iniciar tempranamente y
hacer commits pequeños agreguen funcionalidad incrementalmente.
Se podrán hacer commit hasta el día de la entrega a las 19 hs Arg, luego el sistema automáticamente quitará el acceso
de escritura.

Asi mismo el proyecto debe incluir un pequeño informe en formato Markdown en el README.md del repositorio que contenga una 
explicación del diseño y de las decisiones tomadas para la implementación de la solución.

## Evaluación

### Principios teóricos y corrección de bugs

La evaluación se realizará sobre Github, pudiendo el profesor hacer comentarios en el repositorio y solicitar cambios
o mejoras cuando lo encuentre oportuno, especialmente debido al uso incorrecto de herramientas de concurrencia (por ejemplo presencia de posibles deadlocks).

### Casos de prueba

Se someterá a la aplicación a diferentes casos de prueba que validen la correcta aplicación de las herramientas de concurrencia.

### Informe

El informe debe estar estructurado profesionalmente y debe poder dar cuenta de las decisiones tomadas para implementar la solución.

### Organización del código

El código debe organizarse respetando los criterios de buen diseño y en particular aprovechando las herramientas recomendadas por Rust. Se prohibe el uso de bloques `unsafe`. 

### Tests automatizados

La presencia de tests automatizados que prueben diferentes casos, en especial sobre el uso de las herramientas de concurrencia es un plus.

### Presentación en término

El trabajo deberá entregarse para la fecha estipulada. La presentación fuera de término sin coordinación con antelación con el profesor influye negativamente en la nota final.