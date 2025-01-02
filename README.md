
# ARC Farmework Solana Implementation

This repository contains the implementation of the **Logarithmic Market Scoring Rule (LMSR)** programs, designed to provide dynamic pricing, inherent liquidity, and risk management for decentralized coverage pools.

## Prerequisites

Before you begin, ensure you have the following installed on your system:

- [Rust](https://www.rust-lang.org/tools/install): Install Rust and Cargo.
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools): Install and configure the Solana CLI.
- [Anchor Framework](https://project-serum.github.io/anchor/getting-started/installation.html): Install Anchor via Cargo.
- Node.js and Yarn: Required for Anchor dependencies.

```bash
# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --tag v0.25.0

# Verify installation
anchor --version
```

## Setting Up the Project

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/your-repo/lmsr-anchor.git
   cd lmsr-anchor
   ```

2. **Install Dependencies**:
   Navigate to the project directory and install dependencies.
   ```bash
   yarn install
   ```

3. **Configure Solana CLI**:
   Set up your Solana CLI environment:
   ```bash
   solana config set --url devnet
   solana-keygen new --outfile ~/.config/solana/id.json
   solana airdrop 2
   ```

4. **Build the Program**:
   Compile the LMSR program:
   ```bash
   anchor build
   ```

5. **Deploy the Program**:
   Deploy the compiled LMSR program to the Solana blockchain.
   ```bash
   anchor deploy
   ```

6. **Set Program ID**:
   Update the `Anchor.toml` and `lib.rs` files with the deployed program ID:
   ```bash
   # Replace <PROGRAM_ID> with the actual deployed ID
   solana address -k target/deploy/lmsr-anchor-keypair.json
   ```

   Update `Anchor.toml`:
   ```toml
   [programs.devnet]
   lmsr_anchor = "<PROGRAM_ID>"
   ```

   Update `lib.rs`:
   ```rust
   declare_id!("<PROGRAM_ID>");
   ```

7. **Run Tests**:
   Execute the provided tests to ensure functionality:
   ```bash
   anchor test
   ```

## Project Structure

- `programs/lmsr/src`: Contains the LMSR program logic.
- `tests`: Integration tests for LMSR functionality.
- `Anchor.toml`: Anchor configuration file.
- `migrations/deploy.ts`: Script for program deployment.

## LMSR Program Features

1. **Dynamic Pricing**: Implements the LMSR cost function to calculate dynamic share prices.
2. **Liquidity Management**: Ensures inherent liquidity via the `b` parameter.
3. **Worst-Case Loss Coverage**: Calculates maximum potential loss to ensure market stability.
4. **Share Trading**: Facilitates dynamic share buying and selling based on LMSR odds.
5. **Oracle Integration**: Supports modular oracles for event outcome verification.

## Example Commands

- **Create a Market**:
  Use the client-side interface to create a new market.
  ```bash
  ts-node migrations/create-market.ts
  ```

- **Place a Bet**:
  Interact with the LMSR program to place a bet.
  ```bash
  ts-node migrations/place-bet.ts
  ```

- **Settle a Market**:
  Submit the final market outcome using the oracle integration.
  ```bash
  ts-node migrations/settle-market.ts
  ```

## Contributing

Contributions are welcome! Please follow the [contribution guidelines](CONTRIBUTING.md) and open a pull request with your improvements.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
