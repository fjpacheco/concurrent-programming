# TP1: Técnicas de Programación Concurrente I 

- [Ejecucion del programa, tests y docs.](#ejecucion-del-programa-tests-y-docs)
  - [Ejecución del programa](#ejecución-del-programa)
    - [Ejecución con logs](#ejecución-con-logs)
    - [Formato del archivo de órdenes](#formato-del-archivo-de-órdenes)
    - [Configuración de constantes del programa](#configuración-de-constantes-del-programa)
  - [Ejecución de tests](#ejecución-de-tests)
  - [Generacion de documentacion](#generacion-de-documentacion)
- [Explicación del diseño y de las decisiones tomadas para la implementación de la solución.](#explicación-del-diseño-y-de-las-decisiones-tomadas-para-la-implementación-de-la-solución)
  - [Secciones críticas identificadas y solución propuesta](#secciones-críticas-identificadas-y-solución-propuesta)
  - [Elección del procesamiento de órdenes](#elección-del-procesamiento-de-órdenes)
# Ejecucion del programa, tests y docs.

## Ejecución del programa

Se realiza con `cargo` donde se le puede pasar como argumento el path del archivo de órdenes a procesar.

```bash
cargo run -- <orders.txt>
```

### Ejecución con logs

No era requisito del trabajo práctico, pero para evitar tener un "choclo" de `println!` por todos lados, se utiliza un sistema de log mediante el uso del crate `logs`. Se puede pasar como variable de entorno  `RUST_LOG` el nivel de logs que se quiera ver en la ejecución de la cafetería. Los niveles de logs disponibles son: `trace`, `debug`, `info`, `warn`, `error`. Por default se muestra el nivel de logs `info`, pero si se quisiera ver todo tipo de error se puede ejecutar el programa de la siguiente manera:

```bash
RUST_LOG=trace cargo run
```

### Formato del archivo de órdenes

Dicho archivo de ordenes debe tener el siguiente formato:
```txt
A<cantidad_de_agua_caliente> M<cantidad_de_cafe_molido> C<cantidad_de_cacao E<cantidad_de_espuma_de_leche>
```
Donde la cantidad puede ser float (números con decimales separados por un punto) o entero.

Un ejemplo para dos pedidos, uno que requiera 1 gramo de Agua, 0.5 gramos de cafe molido, 1 gramo de cacao y 2 gramos de espuma de leche. Y otro pedido que requiera 2 gramos de Agua, 2 gramos de café molido, 2 gramos de cacao y 2 gramos de espuma de leche.

```txt
A1 M0.5 C1 E2
A2 M2 C2 E2
```

También se acepta que los pedidos reciban menos de 4 ingredientes, por ejemplo un pedido con solo 1 gramo de Agua y 0.5 gramos de café molido.
```txt
A1 M0.5
```

### Configuración de constantes del programa

Se definen mediante variables de entorno las siguientes constantes del programa que se puede modificar para probar el programa con diferentes dispensers con diferentes capacidades de contenedores.

```txt
C_CACAO = "50.0"
A_AGUA_CALIENTE = "1000.0"
E_ESPUMA_LECHE = "10.0"
M_GRANOS_MOLIDOS = "10000.0"
L_LECHE_FRIA = "1000.0"
G_GRANOS = "1000.0"
N_DISPENSERS = "10.0"
```

Donde cada, expresada en **gramos**, representa:

* `C_CACAO`: Capacidad del contenedor de cacao.
* `A_AGUA_CALIENTE`: Capacidad del contenedor de agua caliente.
* `E_ESPUMA_LECHE`: Capacidad del contenedor de espuma de leche.
* `M_GRANOS_MOLIDOS`: Capacidad del contenedor de cafe molido.
* `L_LECHE_FRIA`: Cantidad de leche fría para recargar el contenedor E_ESPUMA_LECHE.
* `G_GRANOS`: Cantidad de granos para recargar el contenedor M_GRANOS_MOLIDOS.

Y también la cantidad de dispensers a invocar:

* `N_DISPENSERS`: Cantidad de threads dispensers a invocar. Como máximo se puede tener 1024 dispensers (`utils.rs: LIMIT_DISPENSERS`).

Cabe remarcar que todos los 4 diferentes contenedores inician su cantidad con su capacidad máxima.


## Ejecución de tests

Para ejecutar los tests mediante cargo:

```bash
cargo test
```

Cabe mencionar que los tests integrales del sistema completo se encuentran en `cafateria.rs`. 

En este caso, los tests tendrán en cuenta la cfg `#[cfg(test)]` para que no se ejecute ningun el sleep() en el programa usando yield_now(). Esto se define en `lib.rs`. De esta manera se logra testear implícitamente la concurrencia sin la utilización de Loom (intente utilizar Loom pero no logre que funcione correctamente y ya no llegaba con el tiempo para seguir intentando). 

## Generacion de documentacion

Para generar la documentación del proyecto mediante cargo:

```bash
cargo doc --open
```

# Explicación del diseño y de las decisiones tomadas para la implementación de la solución.

## Secciones críticas identificadas y solución propuesta

Hay 4 secciones críticas identificadas para la resolución presentada:

* Los contenedores de los ingredientes (aka `containers: Arc<Conteiners>`)
    * Hay 3 tipos de contenedores (Infinity Conteiner, No Rechargable Conteiner y Rechargable Conteiner) donde cada contenedor implementará el trait de "ApplyConteiner" correspondiente.
    * En la estructura de datos "Conteiners" se tendrán los 4 contenedores (Agua de tipo Infinity Conteiner, Cacao de tipo No Rechargable Conteiner y la Espuma de Leche junto al Café Molido de tipos Rechargable Conteiner) necesarios para la cafetería, donde para cada uno de esos contenedores se almacenará lo que es un Arc Mutex de un objeto que implementa el trait de "ApplyConteiner" correspondiente (en nuestro caso, los contenedores).
    * Los Dispensers podrán acceder a los "Conteiners" mediante un Arc, y cuando alguno Dispenser quiera (y necesite) aplicar un ingrediente, se hará un lock del Mutex del Conteiner correspondiente. 
        * Manejar una estructura con Arc y dentro de la misma los 4 diferentes Arc Mutexs de los Conteiners permite que cada Dispenser pueda acceder a un Conteiner de manera independiente sin bloquear a los demás Dispensers que eventualmente podrían necesitar de otro tipo de Conteiner.

* Los estados de los contenedores (aka `pair_conteiners_states: Arc<(Mutex<ContainersStates>, Condvar)>`)
    * Es la forma de representar los estados de cada Contenedor, será una sección crítica porque cada Dispenser accederá a los estados de los contenedores para saber si puede tomar o no el arc mutex de un Conteiner correspondiente para poder aplicar un ingrediente; y en caso de que pueda tomar el arc mutex del Conteiner correspondiente, el Dispenser setteara el estado del Conteiner como "Taken" en el ARC MUTEX de los "ContainersStates" para que otro Dispenser no pueda tomar el arc mutex del Conteiner correspondiente.
        * Es decir cuando un Dispenser esté aplicando un ingrediente, el mismo antes de aplicar el ingrediente, tomará el arc mutex del estado del Conteiner correspondiente y setteara el Conteiner ha sido tomado (`StateOfConteiner::Taken`) y luego libera el arc mutex para proceder a aplicar el ingrediente. 
            * De esta forma se evita que otro Dispenser pueda acceder al Conteiner-i mientras otro Dispenser está aplicando un ingrediente con el Contener-i.
    * Se utiliza una Condvar para que los Dispensers puedan esperar a que algún Conteiner esté libre para poder aplicar un ingrediente. 
        * En caso de que el Conteiner no esté libre (`StateOfConteiner::Taken`), el Dispenser se bloqueara en la Condvar hasta que el Conteiner esté libre (`StateOfConteiner::Free`).
            * Cuando un Dispenser termine de aplicar un ingrediente, el mismo tomará el arc mutex del estado del Conteiner correspondiente y setteara el Conteiner ha sido liberado (`StateOfConteiner::Free`) y luego notificará a la Condvar para que los Dispensers que estén bloqueados en la misma puedan despertarse y verificar si algún Conteiner que necesita para el pedido está libre (`StateOfConteiner::Free`).
    * Con esta estructura de ConteinersStates facilita el caso cuando un Contenedor no dispone de recursos suficientes para preparar el pedido, pues se tendrá setteado el (`StateOfConteiner::NoEnoughResource`) para dicho Conteiner.
        * Cuando un Dispenser esté esperando por el estado de un Conteiner en la Condvar, e identifique que el Conteiner no dispone de recursos suficientes para preparar el pedido, el Dispenser cancelará el pedido y el dispenser procederá a esperar otro pedido.


* Las órdenes a procesar (aka `pair_vecdeque_orders: Arc<(Mutex<Option<VecDeque<Order>>>, Condvar)>`)
    * Es un modelo de productor-consumidor implementado con Condvars, un productor y N consumidores. 
    * Hay un único productor que será el thread principal que fue encargado de leer las órdenes de un archivo .txt y luego procederá a enviar cada orden insertándose en la cola de órdenes.
    * Habrá `N_DISPENSERS` consumidores que son los threads Dispensers, que estarán esperando por una orden de la cola de órdenes para procesar.
    * Esta cola de órdenes está encapsulada en una Option de Rust debido a que con la misma facilita la representación del caso donde ya el productor no tiene más pedidos que insertar en la cola, y debe avisar a los dispensers que no hay más pedidos para procesar. 
        * Esta "señal" se realiza mandando un None en esta arc mutex. En este arc mutex, el productor insertará este "None" cuando ya no haya más pedidos en la cola (es decir, todos los dispensers tomaron y procesaron todas las órdenes de la cola). Es decir que el productor va hacer un wait sobre el condvar de la cola de ordenes esperando que dicha cola está vacía.
        * Cuando el productor inserte el None: mediante la Condvar va a notificar a todos Dispensers de tal forma que aquellos threads que estaban esperando por un pedido, al recibir un None sabrán que ya no hay más pedidos para procesar y por ende terminan su ejecución.


* Las ordenes procesadas (aka `pair_vecdeque_system_alert: Arc<(Mutex<VecDeque<Order>>, Condvar)>`)
    * Es un modelo de productor-consumidor implementado con Condvars, de N productores y un consumidor.
    * Hay `N_DISPENSERS` productores que son los threads Dispensers, que van a insertar en la cola de órdenes procesadas cada orden que hayan terminado de procesar, así sea una orden que se haya cancelado por falta de recursos o una orden que se haya preparado correctamente.
    * Hay un único consumidor que será el thread de System Alert. Este thread estará esperando en la Codnvar por una orden de la cola de órdenes procesadas. Una vez que este consumidor reciba una orden, el mismo la guardará en un cola interna para que luego periódicamente este mostrando estadísticas en base a las órdenes almacenadas que se hayan recibido.
        * Este consumidor thread system alert finalizará cuando la cantidad de órdenes procesadas que recibe sea igual a la cantidad de órdenes totales que debió mandar el productor del modelo productor-consumidor mencionado en el anterior ítem

## Elección del procesamiento de órdenes

Luego de que un Dispenser reciba un Pedido a procesar (`Dispenser::wait_pedido`), el mismo va a procesar el pedido (`Dispenser::process_order`) de la siguiente forma:
* El dispenser va a esperar en la Condvar de los ConteinersState que al menos algún Conteiner esté libre para poder aplicar un ingrediente del Pedido (`ConteinerStates::order_is_processable`). 
    * En la Condvar también se evalúa el caso donde el pedido requiere un ingrediente que el Conteiner no dispone de recursos suficiente (`ConteinerStates::NoEnoughResource`), en este caso la Condvar dejará de esperar por un Conteiner libre y el Dispenser cancelará el pedido (además de notificar al System Alert del pedido procesado pero no completado) para luego esperar otro pedido.
    
* Cuando el Dispenser encuentre que haya al menos un contenedor libre (indicado en el arc mutex ConteinersState) para el Pedido recibido, se procede a obtener algún contenedor libre indicado en el arc mutex ConteinersState que fue tomado mediante la Condvar. 
    * Cabe resaltar que esta elección de contenedor sobre los contenedores libres (`ConteinerState::find_rng_any_container_free_for`) se realiza de **forma aleatoria**. 
    * Además hay que destacar que cuando la Condvar termina de esperar (salida del wait de la condvar) debido a que aparecio **al menos** un contenedor libre para el pedido: esta "salida del wait de la condvar" es no determinista, pues no se tiene en cuenta alguna decisión sobre qué dispenser-i debe dejar de esperar en al condvar wait sobre los contenedores libres para sus pedidos.
        * Cuando se hace un notify_all sobre esta condvar, se despiertan todos los Dispensers que estaban esperando por un contenedor libre para sus pedidos y el primero que tome el arc mutex de los ConteinersState y encuentre que hay al menos un contenedor libre, será el que procese el pedido y justamente eso es lo no determinístico. No se controla en el código sobre que Dispenser-i debe dejar de esperar en la condvar para que pueda procesar su pedido.
        * Se podría haber implementado algún mecanismo para que el Dispenser-i que haya esperado más tiempo en la condvar sea el que procese su pedido (es decir, salga del wait de la condvar), o algo similar con alguna cola con prioridad, pero se decidió no implementar esto para no agregar más nivel de complejidad al sistema (y además por falta de tiempo).
        * En conclusión, debido a esta elección el sistema no es lo suficientemente **fairness**.

* Cuando se obtiene algún contenedor libre indicado en el arc mutex ConteinersState que fue tomado mediante la Condvar: se proceda a aplicar el ingrediente de ese contenido sobre el pedido (`container_available.apply_ingredient`) pero antes se debe settear en el ConteinersState que dicho contenedor pasará a estar `StateOfConteiner::Taken`. AL settear ese estado (`container_available.set_taken_state()`), mediante el RAII se hara drop del arc mutex del ConteinersState y por ende cualquier otro dispenser podrá acceder a dicho mutex para ver si es posible aplicar otros ingredientes de contenedores que estén libres.
    * Luego de aplicar dicho ingrediente, se volverá a querer tomar lock sobre los ConteinersState, pero esta vez para settear el el estado del contenedor y además de notificar a todos los demás dispensers del estado del contenedor (`container_available.update_and_notify_state`).
        * Dicha función podría settear (en ConteinersState) el estado del contenedor en `StateOfConteiner::NoEnoughResource` o `StateOfConteiner::Free` según como haya terminado el procesamiento del aplicado del ingrediente al pedido.
