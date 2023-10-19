# CryptGain

[StakeAfrik](https://stakeafrik.com) presents CryptGain, a groundbreaking ecosystem designed to revolutionize liquidity provision and PoS blockchain validation and delegation. The project aims to ensure adequate liquidity provision for supported tokens and democratize participation in blockchain validation by enabling individuals passionate about securing PoS blockchains, such as MultiversX, to participate actively without providing a significant initial stake. With a commitment to transparency, innovation, and community engagement, CryptGain introduces a wealth of benefits to its key stakeholders.

![CryptGain Logo](logo.svg)

## Project Overview

CryptGain is more than just a project; it's a thriving community where validators, delegators, and tokens unite to shape the future of blockchain validation and delegation. Whether you're securing the network, actively participating as a delegator, or supporting tokens that back GToken, CryptGain offers a world of opportunities and benefits.

### Unique Value Proposition

#### For Validators:

CryptGain empowers validators with a set of game-changing advantages:

-   **Efficiency:** Validators can focus on securing the network without the costly overhead of acquiring active delegators through advertising campaigns.

-   **Maximized Rewards:** CryptGain's structured incentive mechanisms enable validators to maximize their rewards for safeguarding the network.

-   **Easy Node Management:** Validators can seamlessly use the CryptGain UI to orchestrate on-chain actions to manage their nodes.

#### For Delegators:

CryptGain introduces a world of opportunities for delegators:

-   **Accessible Staking:** Delegators can actively participate in staking on PoS blockchains with little stakes, thanks to CryptGain's innovative liquid staking module. For example, on MultiversX, delegators can use less than 1 $eGLD to stake and receive rewards.

-   **Diverse Incentives:** With the chance to earn cryptERD, cryptgains, Aku NFTs, Uru NFTs, and referral rewards, delegators enjoy a diverse range of incentives.

-   **Long-Term Engagement:** Delegators are encouraged to stay engaged in the ecosystem, thanks to penalties for early undelegation, with the fees distributed back to the community.

-   **Governance:** Using GTokens and Aku NFTs, Delegators can decide which validators would secure the network through the Delegation Smart Contract powering CryptGain.

#### For Tokens Backing GToken:

Tokens backing the GToken benefit from the following:

-   **Increased Utility:** These tokens gain increased utility and demand as the ecosystem's liquidity accelerator module supports them.

-   **Strong Community:** The CryptGain ecosystem fosters a robust and active community, driving interest and engagement with tokens backing the GToken.

-   **Strategic Partnerships:** As CryptGain expands and forms partnerships, the tokens supporting GToken become more diverse, contributing to each token's long-term value and stability.

### Community Engagement

CryptGain thrives on community engagement, driven by three core modules:

1. _**Liquid Staking Module:**_ Delegators are incentivized with cryptERD (CryptGain's version of liquid $eGLD), cryptgains, Aku NFTs, Uru NFTs, and referral rewards for staking their $eGLD and optionally referring others.

1. _**Liquidity Accelerator Module:**_ Users can provide liquidity on xExchange for supported tokens, including those from partner projects, in exchange for GTokens (CryptGain's governance token).

1. _**Governance Module:**_ GToken holders can participate in the governance of CryptGain. They combine their GTokens and Aku NFTs to create voting power for governance, thus influencing the platform's direction. One such use of the voting power is to decide on validators powering the Delegation Smart Contract of the CryptGain platform.

## Smart Contracts

A series of smart contracts powers CryptGain, each playing a vital role in its ecosystem. While we prioritize transparency and encourage open-sourcing, some contracts remain private for obvious reasons. Here, we introduce the contracts integral to CryptGain, including those made public for the xDay 2023 hackathon:

1. **Aku Factory Contract**:

    The Aku Factory contract manages the creation and update of Aku, Uru, and Muo NFTs within the CryptGain ecosystem. These NFTs serve as essential assets with various functions.

2. **Aku-Marketplace Contract**:

    Aku-Marketplace is the first point where newly minted Uru NFTs are traded for GTokens. These NFTs gain power as they age and can be merged with Aku NFTs to enhance their capabilities.

3. **Validator Controller Contract**:

    Manages validator-related functionalities, which include proposing nodes for validation and on-chain management of nodes by validators.

4. **Delegation-Proxy Contract**:

    The Delegation-Proxy contract connects the Liquid Staking and Validator Controller contracts to the Delegation Smart Contract.

5. **Postlaunch Contract:**
   The Postlaunch Contract is the main entry point for delegators to mint, grow, and reduce their Aku, Uru, and Muo NFTs. It also manages the distribution of cryptgains after launch and is an integral part of the Liquid Staking module.

Now, let's introduce the public smart contracts submitted for the hackathon:

6. **GToken Smart Contract**: [Link to README](./g_token/README.md)

    The GToken Smart Contract manages GToken, the governance token of CryptGain, enabling holders to participate actively in platform decision-making and receive incentives for doing so.

7. **Liquid Staking Smart Contract**: [Link to README](./liquid-staking/README.md)

    The Liquid Staking Smart Contract introduces "cryptERD," CryptGain's liquid version of delegated $eGLD. It enables a flexible delegation process for delegators as it allows them to stake sub 1 $eGLD. CryptERD will be part of the tokens used to provide collateral on the GToken Smart Contract

## Roadmap

CryptGain's ambitious roadmap includes expanding beyond MultiversX to bridge native tokens from other PoS blockchains into the MultiversX ecosystem. This venture will involve the development of token bridges and deploying CryptGain on those blockchain networks.

Join us on our journey to transform the blockchain landscape and empower validators, delegators, and tokens within the CryptGain ecosystem. [Join the CryptGain Community](https://t.me/stakeafrik)
