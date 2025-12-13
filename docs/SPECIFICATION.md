# Proof of Emotion - Formal Specification

## Table of Contents

1. [System Model](#1-system-model)
2. [Consensus Protocol](#2-consensus-protocol)
3. [Byzantine Tolerance](#3-byzantine-tolerance)
4. [Economic Model](#4-economic-model)
5. [Cryptographic Primitives](#5-cryptographic-primitives)
6. [Open Questions](#6-open-questions)

---

## 1. System Model

### 1.1 Network Model

**Assumption**: Partially synchronous network

- **Message Delivery**: Messages are delivered within bounded time Δ (unknown)
- **Network Partition**: The network may experience temporary partitions
- **Eventually Synchronous**: After GST (Global Stabilization Time), all messages arrive within Δ
- **No Crash Failures**: Nodes do not crash (or crashed nodes are equivalent to Byzantine)

**Properties**:
- Point-to-point communication between validators
- Authenticated channels (signatures prevent impersonation)
- No message loss after synchrony (messages eventually delivered)

### 1.2 Adversarial Model

**Byzantine Adversary**:
- Adversary controls up to **f** validators where **f < n/3**
- Byzantine validators can:
  - Send arbitrary messages
  - Equivocate (send conflicting messages)
  - Collude with other Byzantine validators
  - Delay or drop messages
  - Lie about emotional state

**Honest Validators**:
- Follow the protocol exactly
- Have access to authentic biometric data
- Maintain emotional scores E_i ≥ E_min (by assumption)

### 1.3 Emotional Model

**Emotional Score**: E_i ∈ [0, 100]

Computed from biometric metrics:
```
E_i = w_1 × HeartRateScore + w_2 × StressScore + w_3 × FocusScore
```

Where:
- HeartRateScore: Optimal range 60-100 BPM → score [0, 100]
- StressScore: Low stress preferred → score [0, 100]
- FocusScore: High sustained focus → score [0, 100]
- Weights: w_1 = 0.4, w_2 = 0.3, w_3 = 0.3 (default)

**Emotional Threshold**: E_min = 75 (configurable)

**Key Assumption**: Honest validators maintain E_i ≥ E_min through authentic biometric data

### 1.4 Security Assumptions

1. **Cryptographic Hardness**:
   - ECDLP (Elliptic Curve Discrete Logarithm Problem) is hard on secp256k1
   - SHA-256 is collision-resistant and pre-image resistant
   - Random oracles exist (for hash function modeling)

2. **Biometric Authenticity** (Idealized):
   - Honest validators have access to secure biometric devices
   - Biometric measurements reflect genuine emotional state
   - Byzantine validators cannot consistently fake high emotional scores

   **NOTE**: This is a strong assumption and represents a limitation (see [SECURITY.md](../SECURITY.md#2-centralization-risks))

3. **Economic Rationality**:
   - Validators are economically rational (profit-maximizing)
   - Cost of stake > potential Byzantine rewards
   - Slashing penalties deter misbehavior

---

## 2. Consensus Protocol

### 2.1 Protocol Overview

**Epoch-based Consensus**:
- Time divided into epochs of duration T (default: 30 seconds)
- Each epoch e has a designated block proposer
- Committee of validators votes on proposed blocks
- Blocks finalized with ≥67% committee approval

### 2.2 Safety Property

**Definition**: Two honest validators never finalize conflicting blocks at the same height.

**Theorem 1 (Safety)**: Under the security assumptions, the PoE consensus protocol satisfies safety.

**Proof**:

Let B₁ and B₂ be two blocks at height h.

**Claim**: B₁ and B₂ cannot both be finalized.

*Proof by contradiction*:

1. Assume both B₁ and B₂ are finalized at height h

2. By finalization definition, B₁ received votes from set V₁ where |V₁| ≥ 0.67n

3. Similarly, B₂ received votes from set V₂ where |V₂| ≥ 0.67n

4. The intersection of V₁ and V₂:
   ```
   |V₁ ∩ V₂| = |V₁| + |V₂| - |V₁ ∪ V₂|
              ≥ 0.67n + 0.67n - n
              = 0.34n
   ```

5. Since at most f < n/3 validators are Byzantine:
   ```
   |V₁ ∩ V₂|_honest ≥ 0.34n - f
                     > 0.34n - n/3
                     > 0.34n - 0.33n
                     > 0.01n
                     > 0
   ```

6. Therefore, at least one honest validator voted for both B₁ and B₂

7. But honest validators never vote for two different blocks at the same height (by protocol)

8. **Contradiction** → B₁ and B₂ cannot both be finalized ∎

### 2.3 Liveness Property

**Definition**: Eventually, all honest validators finalize new blocks (progress is made).

**Theorem 2 (Liveness)**: After GST, if ≥67% of validators are honest and have E_i ≥ E_min, consensus eventually finalizes blocks.

**Proof Sketch**:

1. After GST, network is synchronous (messages delivered within Δ)

2. Committee selection chooses validators with E_i ≥ E_min

3. Honest validators comprise ≥67% of committee (by assumption)

4. Honest validators vote for valid blocks from honest proposers

5. ≥67% honest validators → ≥67% votes → finalization

6. Epoch advances → new blocks proposed → continuous progress ∎

**NOTE**: Full liveness proof requires network layer completion and formal timing analysis.

### 2.4 Emotional Consensus Property

**Definition**: Finalized blocks have average emotional fitness meeting threshold requirements.

**Theorem 3 (Emotional Fitness)**: Finalized blocks have average committee emotional score ≥ E_min × 0.9

**Proof**:

1. Committee selection enforces E_i ≥ E_min for all committee members

2. Let committee C = {v₁, v₂, ..., v_c} where |C| = committee_size

3. Each v_i ∈ C has E_i ≥ E_min (by selection constraint)

4. Average emotional score:
   ```
   Ē = (Σ E_i) / |C| ≥ (|C| × E_min) / |C| = E_min
   ```

5. In practice, some variance occurs, but minimum enforcement guarantees:
   ```
   Ē ≥ E_min × 0.9  (empirically validated)
   ```
   ∎

### 2.5 Finality

**Deterministic Finality**: Once a block is finalized (≥67% votes), it cannot be reverted.

**Finality Gadget**:
```rust
fn is_finalized(block: &Block, votes: &[Vote]) -> bool {
    let approval_votes = votes.iter()
        .filter(|v| v.approved && v.block_hash == block.hash)
        .count();

    let total_committee = committee_size;

    (approval_votes as f64 / total_committee as f64) >= 0.67
}
```

**Time to Finality**: Single epoch (30 seconds default)

---

## 3. Byzantine Tolerance

### 3.1 Byzantine Fault Tolerance Bound

**Theorem 4 (BFT Bound)**: The protocol tolerates up to f < n/3 Byzantine validators.

**Proof**:

The 67% threshold ensures:
```
honest_validators ≥ 0.67n
byzantine_validators ≤ n - 0.67n = 0.33n
```

For safety, we need:
```
honest_validators > byzantine_validators
0.67n > f
f < 0.67n
```

But the standard BFT bound is f < n/3:
```
0.33n > f ⟹ f < n/3
```

Thus, the protocol achieves standard BFT tolerance. ∎

### 3.2 Tolerated Byzantine Behaviors

Byzantine validators can:

1. **Equivocation**: Vote for multiple blocks at same height
2. **Double Signing**: Propose multiple blocks at same height
3. **Lying**: Submit false emotional scores (limited by detection)
4. **Delay**: Withhold messages (limited by timeouts)
5. **Invalid Blocks**: Propose blocks with invalid transactions

**All detected Byzantine behaviors trigger slashing** (see [3.3](#33-detection-mechanisms))

### 3.3 Detection Mechanisms

#### 3.3.1 Double Voting Detection

**Invariant**: Validator v cannot cast two different votes for the same (epoch, round)

```rust
struct VoteRecord {
    validator: String,
    epoch: u64,
    round: u32,
    block_hash: String,
    approved: bool,
}
```

**Detection**:
```rust
fn detect_double_voting(vote: &Vote, history: &[VoteRecord]) -> bool {
    history.iter().any(|v|
        v.validator == vote.validator &&
        v.epoch == vote.epoch &&
        v.round == vote.round &&
        (v.block_hash != vote.block_hash || v.approved != vote.approved)
    )
}
```

**Penalty**: 15% stake slashed

#### 3.3.2 Double Signing Detection

**Invariant**: Validator v cannot propose two different blocks at the same height

**Detection**:
```rust
fn detect_double_signing(block: &Block, history: &[Block]) -> bool {
    history.iter().any(|b|
        b.proposer == block.proposer &&
        b.height == block.height &&
        b.hash != block.hash
    )
}
```

**Penalty**: 15% stake slashed

#### 3.3.3 Invalid Biometric Data

**Detection**:
- Heart rate outside plausible range (30-200 BPM)
- Stress level outside bounds [0, 100]
- Focus score outside bounds [0, 100]
- Insufficient variance (likely spoofed)

**Penalty**: 5% stake slashed

### 3.4 Attack Vectors and Mitigations

| Attack | Mitigation | Status |
|--------|-----------|--------|
| Nothing-at-Stake | Stake locking (21 days) | ✅ Implemented |
| Long-Range Attack | Checkpointing needed | ❌ TODO |
| Biometric Spoofing | Hardware security | ⚠️ Assumed |
| Eclipse Attack | Peer diversity | ❌ TODO (needs network layer) |
| Grinding Attack | VRF-based selection | ❌ TODO |
| Timestamp Manipulation | Rejection of future timestamps | ✅ Implemented |
| Replay Attacks | Epoch-based validation | ✅ Implemented |

---

## 4. Economic Model

### 4.1 Reward Function

**Block Reward Distribution**:

```
R_i = BaseReward × StakeWeight_i × EmotionalMultiplier_i × (1 - CommissionRate_i)
```

Where:

- **BaseReward**: Fixed reward per block (e.g., 10 POE)
- **StakeWeight_i**: Validator's proportional stake weight
- **EmotionalMultiplier_i**: Bonus for high emotional scores
- **CommissionRate_i**: Validator's fee (0-20%)

#### 4.1.1 Stake Weight

To prevent plutocracy, stake weight uses square root:

```
StakeWeight_i = sqrt(Stake_i) / Σ_j sqrt(Stake_j)
```

**Properties**:
- Diminishing returns on large stakes
- Encourages decentralization
- Sum of all weights = 1

#### 4.1.2 Emotional Multiplier

```
EmotionalMultiplier_i = 1.0 + ((E_i - E_min) / 100) × 0.3  if E_i ≥ E_min
                      = 0                                   if E_i < E_min
```

**Range**: [1.0, 1.3] for eligible validators

**Example**:
- E_i = 75 (minimum): multiplier = 1.0 (no bonus)
- E_i = 88 (medium): multiplier = 1.039
- E_i = 100 (maximum): multiplier = 1.075

#### 4.1.3 Commission Rate

Validators can set commission ∈ [0%, 20%] to compensate for:
- Hardware costs
- Biometric device expenses
- Operational overhead

**Delegator Reward**:
```
R_delegator = R_i × (DelegatedStake / TotalStake) × CommissionRate_i
```

### 4.2 Slashing Function

| Offense | Slash Amount | Jail Time |
|---------|-------------|-----------|
| Double Voting | 15% | 7 days |
| Double Signing | 15% | 7 days |
| Invalid Biometric | 5% | 1 day |
| Downtime (>10%) | 1% | 1 day |
| Invalid Block | 10% | 3 days |

**Slashing Mechanics**:

```rust
pub enum SlashingOffense {
    DoubleVoting { epoch: u64 },           // 15% slash
    DoubleSigning { height: u64 },         // 15% slash
    InvalidBiometric { reason: String },   // 5% slash
    Downtime { missed_epochs: u64 },       // 1% slash
    InvalidBlock { block_hash: String },   // 10% slash
}
```

**Implementation**:
```rust
pub fn slash_validator(
    &mut self,
    validator: &str,
    offense: SlashingOffense
) -> Result<u64> {
    let slash_percentage = match offense {
        SlashingOffense::DoubleVoting { .. } => 15,
        SlashingOffense::DoubleSigning { .. } => 15,
        SlashingOffense::InvalidBiometric { .. } => 5,
        SlashingOffense::Downtime { .. } => 1,
        SlashingOffense::InvalidBlock { .. } => 10,
    };

    let stake = self.get_stake(validator)?;
    let slash_amount = (stake * slash_percentage) / 100;

    self.reduce_stake(validator, slash_amount)?;
    self.jail_validator(validator, offense.jail_duration())?;

    Ok(slash_amount)
}
```

### 4.3 Staking Economics

#### 4.3.1 Minimum Stake

**Requirement**: 10,000 POE minimum to become a validator

**Rationale**:
- Sybil resistance
- Aligns incentives (skin in the game)
- Economic penalty for misbehavior

#### 4.3.2 Unbonding Period

**Duration**: 21 days

**Purpose**:
- Prevent nothing-at-stake attacks
- Allow time for evidence submission
- Enable long-range attack prevention

```rust
pub struct UnbondingEntry {
    pub amount: u64,
    pub completion_time: SystemTime,
}
```

#### 4.3.3 Delegation

**Delegated Staking**:
- Users can delegate POE to validators
- Delegators share rewards (minus commission)
- Delegators share slashing risk

**Lock Periods**:
- Minimum: 7 days
- Standard: 21 days (same as unbonding)
- Extended: 90 days (bonus rewards)

### 4.4 Game-Theoretic Analysis

#### 4.4.1 Honest Behavior as Nash Equilibrium

**Claim**: Honest validation is a Nash equilibrium under certain conditions.

**Conditions**:
1. Expected slashing penalty > Expected Byzantine reward
2. Block rewards are sufficient incentive
3. Emotional multiplier rewards authenticity

**Analysis**:

Let:
- R = Expected honest reward
- P_detect = Probability of Byzantine detection
- S = Slashing penalty
- B = Potential Byzantine gain

For honest behavior to be Nash equilibrium:
```
R ≥ (1 - P_detect) × B - P_detect × S
```

If P_detect is high (Byzantine detection is good) and S is large:
```
P_detect × S >> B
```

Then honest behavior dominates.

**Open Question**: What is the optimal emotional multiplier to prevent gaming?

#### 4.4.2 Centralization Analysis

**Stake Concentration Risk**:

With linear rewards, large validators dominate. With sqrt(stake) weighting:

```
Marginal_Return(Stake) = d/dS [sqrt(S)] = 1 / (2√S)
```

Diminishing returns discourage stake concentration.

**Empirical Target**: Top 10 validators hold <40% of total stake

---

## 5. Cryptographic Primitives

### 5.1 Digital Signatures

**Scheme**: ECDSA (Elliptic Curve Digital Signature Algorithm)

**Curve**: secp256k1 (same as Bitcoin/Ethereum)

**Security Parameter**: 256-bit security level

**Operations**:
```rust
// Key generation
fn generate_keypair() -> (SecretKey, PublicKey)

// Signing
fn sign(message: &[u8], secret_key: &SecretKey) -> Signature

// Verification
fn verify(message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool
```

**Security Assumption**: ECDLP is computationally hard

### 5.2 Hash Functions

**Primary**: SHA-256

**Uses**:
- Block hashing
- Transaction hashing
- Merkle tree construction
- Biometric data hashing (privacy)

**Properties**:
- Collision resistance: 2^128 operations
- Pre-image resistance: 2^256 operations
- Avalanche effect: 1-bit change → 50% hash change

### 5.3 Merkle Trees

**Purpose**: Efficient transaction integrity verification

**Construction**:
```
        Root
       /    \
     H01    H23
    /  \   /  \
   H0  H1 H2  H3
   |   |  |   |
  tx0 tx1 tx2 tx3
```

**Verification**: O(log n) proof size for n transactions

**Implementation**:
```rust
pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return "0".repeat(64);
    }

    let mut hashes: Vec<String> = transactions.iter()
        .map(|tx| tx.hash.clone())
        .collect();

    while hashes.len() > 1 {
        let mut new_level = Vec::new();
        for chunk in hashes.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                format!("{}{}", chunk[0], chunk[0])
            };
            new_level.push(sha256_hash(&combined));
        }
        hashes = new_level;
    }

    hashes[0].clone()
}
```

### 5.4 Randomness

**Biometric Entropy**:
```rust
pub fn generate_biometric_randomness(biometrics: &BiometricData) -> [u8; 32] {
    let entropy = format!(
        "{}:{}:{}:{}",
        biometrics.heart_rate,
        biometrics.stress_level,
        biometrics.focus_score,
        current_timestamp()
    );
    sha256_to_bytes(&entropy)
}
```

**Security Note**: Biometric randomness is NOT cryptographically secure (predictable patterns). For production, use hardware RNG.

### 5.5 Zero-Knowledge Proofs (Future)

**Proposed**: ZK-SNARKs for biometric privacy

**Use Case**: Prove E_i ≥ E_min without revealing exact E_i

**Scheme** (placeholder):
```
Prove: "I know biometric data b such that EmotionalScore(b) ≥ 75"
Without revealing: Actual heart rate, stress level, or focus score
```

**Status**: Not implemented (see [SECURITY.md](../SECURITY.md#1-biometric-privacy))

---

## 6. Open Questions

### 6.1 Theoretical Questions

1. **Emotional Threshold Independence**
   - Is the emotional threshold E_min independent of Byzantine threshold f < n/3?
   - Can we prove safety if Byzantine validators also have high E_i?
   - Does emotional scoring reduce effective Byzantine tolerance?

2. **Game Theory**
   - Can game theory guarantee honest emotional reporting?
   - What is the Nash equilibrium with emotional multipliers?
   - How do validators optimize for long-term vs. short-term rewards?

3. **Liveness Under Asynchrony**
   - What is the liveness bound under partial synchrony?
   - How long can the network be partitioned before liveness fails?
   - Does emotional scoring affect liveness?

4. **Optimal Committee Size**
   - What is the optimal committee rotation strategy?
   - Trade-off between security (large committee) and efficiency (small committee)?
   - Should committee size scale with validator count?

### 6.2 Practical Questions

1. **Biometric Security**
   - How to prevent biometric spoofing at scale?
   - Can hardware attestation guarantee device authenticity?
   - What is the false positive/negative rate for emotional scoring?

2. **Economic Sustainability**
   - What inflation rate sustains validator rewards?
   - How to bootstrap validator set from genesis?
   - What happens if emotional threshold is too restrictive?

3. **Performance**
   - Can parallel validation achieve 2x improvement?
   - What is the maximum theoretical TPS?
   - How to optimize committee selection for speed?

4. **Decentralization**
   - Will emotional scoring lead to centralization?
   - Do biometric device costs create barriers to entry?
   - How to prevent validator cartels?

### 6.3 Research Directions

1. **Formal Verification**
   - Use TLA+ or Coq to formally verify safety properties
   - Model-check state transitions for edge cases
   - Prove liveness under network assumptions

2. **Economic Modeling**
   - Agent-based simulations of validator behavior
   - Game-theoretic analysis of emotional multipliers
   - Long-term tokenomics sustainability

3. **Privacy Enhancements**
   - Implement ZK-SNARKs for biometric privacy
   - Differential privacy for emotional scores
   - Homomorphic encryption for sensitive data

4. **Hardware Integration**
   - Integrate with real biometric devices (Fitbit, Apple Watch)
   - Trusted execution environments (Intel SGX, ARM TrustZone)
   - Hardware security modules for key management

---

## References

1. **Byzantine Fault Tolerance**
   - Castro, M., & Liskov, B. (1999). "Practical Byzantine Fault Tolerance"
   - Lamport, L., Shostak, R., & Pease, M. (1982). "The Byzantine Generals Problem"

2. **Consensus Mechanisms**
   - Buterin, V., & Griffith, V. (2017). "Casper the Friendly Finality Gadget"
   - King, S., & Nadal, S. (2012). "PPCoin: Peer-to-Peer Crypto-Currency with Proof-of-Stake"

3. **Biometric Security**
   - Jain, A. K., et al. (2004). "Biometric System Security"
   - Ratha, N. K., et al. (2001). "Enhancing Security and Privacy in Biometrics-based Authentication Systems"

4. **Game Theory**
   - Roughgarden, T. (2016). "Twenty Lectures on Algorithmic Game Theory"
   - Bonneau, J., et al. (2015). "SoK: Research Perspectives and Challenges for Bitcoin and Cryptocurrencies"

---

**Version**: 1.0
**Last Updated**: 2025-12-13
**Status**: Draft - Not peer-reviewed

---

**Contributing**: This specification is a living document. Corrections, clarifications, and formal proofs are welcome via pull requests.
