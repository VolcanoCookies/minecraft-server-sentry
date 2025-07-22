// Wait for macro for unions of packets
macro_rules! wait_for {
    ($read:expr, $($t:ident)|+ , |$var:ident| $body:expr) => {{
        // Local union enum
        paste::paste! {
            #[derive(Debug)]
            pub enum __WaitForUnion {
                $(
                    [<$t Var>]($t),
                )+
            }
        }

        use __WaitForUnion::*;

        let $var = loop {
            let raw = $read.wait_for_raw().await?;
            if let Some(descriptor) = raw.descriptor {
                paste::paste! {
                    $(
                        if *descriptor == $t::PACKET_DESCRIPTOR {
                            let packet: $t = packet::read::PacketReadable::read(&mut raw.bytes.as_slice())?;
                            break __WaitForUnion::[<$t Var>](packet);
                        }
                    )+
                }
            }
        };

        $body
    }};
}
