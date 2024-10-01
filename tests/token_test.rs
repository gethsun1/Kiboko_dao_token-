#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::signature::Signer;

    #[tokio::test]
    async fn test_create_token() {
        let mut program_test = ProgramTest::new(
            "kiboko_dao_token", // Run your program
            id(),               // Program ID
            processor!(process_instruction),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Test token creation logic
        let mint_pubkey = create_mint(&mut banks_client, &payer, &recent_blockhash, 9).await;

        assert!(mint_pubkey.is_some());
    }
}
