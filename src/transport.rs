#[cfg(test)]
mod tests {
    use crate::avp;
    use crate::avp::enumerated::EnumeratedAvp;
    use crate::avp::identity::IdentityAvp;
    use crate::avp::unsigned32::Unsigned32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::Avp;
    use crate::client::DiameterClient;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};
    use crate::error::Error;
    use crate::server::DiameterServer;

    #[tokio::test]
    async fn test_diameter_transport() {
        // Diameter Server
        tokio::spawn(async move {
            let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();
            server
                .handle(|req| -> Result<DiameterMessage, Error> {
                    println!("Request : {}", req);

                    let mut res = DiameterMessage::new(
                        CommandCode::CreditControl,
                        ApplicationId::CreditControl,
                        0,
                        1123158610,
                        3102381851,
                    );
                    res.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
                    res.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
                    res.add_avp(avp!(263, None, UTF8StringAvp::new("ses;123458890"), true));
                    res.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
                    res.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
                    res.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));
                    Ok(res)
                })
                .await
                .unwrap();
        });

        // Diameter Client
        let mut client = DiameterClient::new();
        let _ = client.connect("localhost:3868").await;

        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            REQUEST_FLAG,
            1123158610,
            3102381851,
        );
        ccr.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
        ccr.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
        ccr.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
        ccr.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
        ccr.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
        let cca = client.send(ccr).await.unwrap();

        println!("Response: {}", cca);

        // Assert Result-Code
        let result_code = &cca.get_avp(268).unwrap();
        assert_eq!(result_code.get_unsigned32().unwrap(), 2001);
    }
}
