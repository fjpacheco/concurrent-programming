use actix::Message;

#[derive(Debug, Clone, Copy)]
pub enum MensajeCafetera {
    SUMAR = 0,
    RESTAR,
    PING,
    OKEY,
    ERROR,
    DESCONECTAR,
    CONECTAR,
    DESCONOCIDO,
}

/// Mensajes que serán enviados entre las Cafeteras y su correspondiente Nodo
impl MensajeCafetera {
    pub fn from_bytes(byte: u8) -> MensajeCafetera {
        match byte {
            0_u8 => MensajeCafetera::SUMAR,
            1_u8 => MensajeCafetera::RESTAR,
            2_u8 => MensajeCafetera::PING,
            3_u8 => MensajeCafetera::OKEY,
            4_u8 => MensajeCafetera::ERROR,
            5_u8 => MensajeCafetera::DESCONECTAR,
            6_u8 => MensajeCafetera::CONECTAR,
            _ => MensajeCafetera::DESCONOCIDO,
        }
    }
}

pub trait MensajeCafeteraBytes {
    fn new(id_cafetera: u8, id_cuenta: u32, cantidad_modificar: u32) -> Self;
    fn get_tipo_mensaje(&self) -> u8;
    fn get_id_cafetera(&self) -> u8;
    fn get_id_cuenta(&self) -> u32;
    fn get_cantidad_modificar(&self) -> u32;

    fn to_bytes(&self) -> Vec<u8> {
        let result = vec![self.get_tipo_mensaje(), self.get_id_cafetera()];
        let id_cuenta_bytes = self.get_id_cuenta().to_be_bytes().to_vec();
        let cantidad_bytes = self.get_cantidad_modificar().to_be_bytes().to_vec();
        [result, id_cuenta_bytes, cantidad_bytes].concat()
    }

    fn from_bytes(bytes: Vec<u8>) -> Self
    where
        Self: Sized,
    {
        let id_cafetera = bytes[1];
        let id_cuenta = u32::from_be_bytes(
            bytes[2..6]
                .try_into()
                .expect("Siempre se mandan los bytes correctos"),
        );
        let cantidad_modificar = u32::from_be_bytes(
            bytes[6..10]
                .try_into()
                .expect("Siempre se mandan los bytes correctos"),
        );
        Self::new(id_cafetera, id_cuenta, cantidad_modificar)
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Representa un nuevo pedido que suma creditos en una tarjeta
pub struct Sumar {
    /// Tipo de mensaje (sumar)
    pub tipo_mensaje: u8,
    /// id correspondiente a la cafetera que inicio el pedido
    pub id_cafetera: u8,
    /// id de la tarjeta del usuario
    pub id_cuenta: u32,
    /// cantidad de creditos a ser sumados
    pub cantidad_modificar: u32,
}

impl MensajeCafeteraBytes for Sumar {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_cantidad_modificar(&self) -> u32 {
        self.cantidad_modificar
    }
    fn new(id_cafetera: u8, id_cuenta: u32, cantidad_modificar: u32) -> Sumar {
        Sumar {
            tipo_mensaje: 0,
            id_cafetera,
            id_cuenta,
            cantidad_modificar,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Representa un nuevo pedido que resta creditos en una tarjeta
pub struct Restar {
    /// tipo de mensaje (restar)
    pub tipo_mensaje: u8,
    /// id correspondiente a la cafetera que inicio el pedido
    pub id_cafetera: u8,
    /// id de la tarjeta del usuario
    pub id_cuenta: u32,
    /// cantidad de creditos a ser restados
    pub cantidad_modificar: u32,
}

impl MensajeCafeteraBytes for Restar {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_cantidad_modificar(&self) -> u32 {
        self.cantidad_modificar
    }
    fn new(id_cafetera: u8, id_cuenta: u32, cantidad_modificar: u32) -> Restar {
        Restar {
            tipo_mensaje: 1,
            id_cafetera,
            id_cuenta,
            cantidad_modificar,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Representa un mensaje que se envia de la cafetera al nodo en caso de timeout
pub struct Ping {
    /// tipo de mensaje (ping)
    pub tipo_mensaje: u8,
    /// id de la cafetera que envia el mensaje
    pub id_cafetera: u8,
}

impl MensajeCafeteraBytes for Ping {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        0
    }
    fn get_cantidad_modificar(&self) -> u32 {
        0
    }
    fn new(id_cafetera: u8, _id_cuenta: u32, _cantidad_modificar: u32) -> Ping {
        Ping {
            tipo_mensaje: 2,
            id_cafetera,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Representa un mensaje de Ok tanto de la cafetera al nodo, para confirmar la
/// preparacion de un cafe, como de un nodo a cafetera para confirmar alguna etapa del pedido
pub struct OkeyToCafetera {
    /// tipo de mensaje (OkeyToCafetera)
    pub tipo_mensaje: u8,
    /// id de la cafetera que inciio el pedido
    pub id_cafetera: u8,
    /// id de la cuenta del usuario
    pub id_cuenta: u32,
}

impl MensajeCafeteraBytes for OkeyToCafetera {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_cantidad_modificar(&self) -> u32 {
        0
    }
    fn new(id_cafetera: u8, id_cuenta: u32, _cantidad_modificar: u32) -> OkeyToCafetera {
        OkeyToCafetera {
            tipo_mensaje: 3,
            id_cafetera,
            id_cuenta,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje que representa un error, puede ser en la preparación de un cafe (si se envia de la cafetera al nodo)
/// o puede representar falta de saldo, falla en la transacción si se envia del nodo a la cafetera
pub struct Error {
    /// tipo de mensaje (error)
    pub tipo_mensaje: u8,
    /// id de la cafetera que inicio el pedido
    pub id_cafetera: u8,
    /// id de la cuenta de usuario
    pub id_cuenta: u32,
}

impl MensajeCafeteraBytes for Error {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        self.id_cuenta
    }
    fn get_cantidad_modificar(&self) -> u32 {
        0
    }
    fn new(id_cafetera: u8, id_cuenta: u32, _cantidad_modificar: u32) -> Error {
        Error {
            tipo_mensaje: 4,
            id_cafetera,
            id_cuenta,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje que es enviado por el proceso "desconexion" que avisa a un nodo
/// que se desconecto a la red.
pub struct Desconectar {
    /// tipo de mensaje (Desconectar)
    pub tipo_mensaje: u8,
    pub id_cafetera: u8,
}

impl MensajeCafeteraBytes for Desconectar {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        0
    }
    fn get_cantidad_modificar(&self) -> u32 {
        0
    }
    fn new(id_cafetera: u8, _id_cuenta: u32, _cantidad_modificar: u32) -> Desconectar {
        Desconectar {
            tipo_mensaje: 5,
            id_cafetera,
        }
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
/// Mensaje que es enviado por el proceso "desconexion" que avisa a un nodo
/// que se volvio a conectar a la red.
pub struct Conectar {
    pub tipo_mensaje: u8,
    pub id_cafetera: u8,
}

impl MensajeCafeteraBytes for Conectar {
    fn get_tipo_mensaje(&self) -> u8 {
        self.tipo_mensaje
    }
    fn get_id_cafetera(&self) -> u8 {
        self.id_cafetera
    }
    fn get_id_cuenta(&self) -> u32 {
        0
    }
    fn get_cantidad_modificar(&self) -> u32 {
        0
    }
    fn new(id_cafetera: u8, _id_cuenta: u32, _cantidad_modificar: u32) -> Conectar {
        Conectar {
            tipo_mensaje: 6,
            id_cafetera,
        }
    }
}

#[cfg(test)]
mod mensajes_cafetera_test {
    use crate::mensajes_cafetera::{Error, OkeyToCafetera, Ping, Restar};

    use super::{MensajeCafeteraBytes, Sumar};

    #[test]
    fn sumar_to_bytes() {
        let test_pkt = Sumar::new(10, 3, 100);
        let expected = vec![0, 10, 0, 0, 0, 3, 0, 0, 0, 100];

        assert_eq!(expected, test_pkt.to_bytes())
    }

    #[test]
    fn sumar_from_bytes() {
        let expected = Sumar::new(100, 5, 50);
        let bytes = vec![0, 100, 0, 0, 0, 5, 0, 0, 0, 50];
        let final_pkt = Sumar::from_bytes(bytes);

        assert_eq!(expected.tipo_mensaje, final_pkt.tipo_mensaje);
        assert_eq!(expected.id_cafetera, final_pkt.id_cafetera);
        assert_eq!(expected.id_cuenta, final_pkt.id_cuenta);
        assert_eq!(expected.cantidad_modificar, final_pkt.cantidad_modificar)
    }

    #[test]
    fn restar_to_bytes() {
        let test_pkt = Restar::new(10, 3, 100);
        let expected = vec![1, 10, 0, 0, 0, 3, 0, 0, 0, 100];

        assert_eq!(expected, test_pkt.to_bytes())
    }

    #[test]
    fn restar_from_bytes() {
        let expected = Restar::new(100, 5, 50);
        let bytes = vec![1, 100, 0, 0, 0, 5, 0, 0, 0, 50];
        let final_pkt = Restar::from_bytes(bytes);

        assert_eq!(expected.tipo_mensaje, final_pkt.tipo_mensaje);
        assert_eq!(expected.id_cafetera, final_pkt.id_cafetera);
        assert_eq!(expected.id_cuenta, final_pkt.id_cuenta);
        assert_eq!(expected.cantidad_modificar, final_pkt.cantidad_modificar)
    }

    #[test]
    fn ping_to_bytes() {
        let test_pkt = Ping::new(10, 3, 100);
        let expected = vec![2, 10, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(expected, test_pkt.to_bytes())
    }

    #[test]
    fn ping_from_bytes() {
        let expected = Ping::new(100, 5, 50);
        let bytes = vec![2, 100, 0, 0, 0, 0, 0, 0, 0, 0];
        let final_pkt = Ping::from_bytes(bytes);

        assert_eq!(expected.tipo_mensaje, final_pkt.tipo_mensaje);
        assert_eq!(expected.id_cafetera, final_pkt.id_cafetera);
    }

    #[test]
    fn okey_to_bytes() {
        let test_pkt = OkeyToCafetera::new(10, 3, 100);
        let expected = vec![3, 10, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(expected, test_pkt.to_bytes())
    }

    #[test]
    fn okey_from_bytes() {
        let expected = OkeyToCafetera::new(100, 5, 50);
        let bytes = vec![3, 100, 0, 0, 0, 0, 0, 0, 0, 0];
        let final_pkt = OkeyToCafetera::from_bytes(bytes);

        assert_eq!(expected.tipo_mensaje, final_pkt.tipo_mensaje);
        assert_eq!(expected.id_cafetera, final_pkt.id_cafetera);
    }

    #[test]
    fn error_to_bytes() {
        let test_pkt = Error::new(10, 3, 100);
        let expected = vec![4, 10, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(expected, test_pkt.to_bytes())
    }

    #[test]
    fn error_from_bytes() {
        let expected = Error::new(100, 5, 50);
        let bytes = vec![4, 100, 0, 0, 0, 0, 0, 0, 0, 0];
        let final_pkt = Error::from_bytes(bytes);

        assert_eq!(expected.tipo_mensaje, final_pkt.tipo_mensaje);
        assert_eq!(expected.id_cafetera, final_pkt.id_cafetera);
    }
}
