/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/prediction_market.json`.
 */
export type PredictionMarket = {
  "address": "AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ",
  "metadata": {
    "name": "predictionMarket",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "buyShares",
      "discriminator": [
        40,
        239,
        138,
        154,
        8,
        37,
        106,
        108
      ],
      "accounts": [
        {
          "name": "market",
          "writable": true
        },
        {
          "name": "buyerTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "baseTokenMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "marketTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "market"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "baseTokenMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "baseTokenMint"
        },
        {
          "name": "outcomeMint",
          "writable": true
        },
        {
          "name": "buyerShareAccount",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "outcomeIndex",
          "type": "u64"
        },
        {
          "name": "numShares",
          "type": "u64"
        }
      ]
    },
    {
      "name": "createMarket",
      "discriminator": [
        103,
        226,
        97,
        235,
        200,
        188,
        251,
        254
      ],
      "accounts": [
        {
          "name": "market",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  97,
                  114,
                  107,
                  101,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "marketId"
              }
            ]
          }
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "baseTokenMint"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "marketId",
          "type": "u64"
        },
        {
          "name": "title",
          "type": "string"
        },
        {
          "name": "outcomes",
          "type": {
            "vec": "string"
          }
        },
        {
          "name": "oracle",
          "type": "pubkey"
        },
        {
          "name": "b",
          "type": "u64"
        },
        {
          "name": "duration",
          "type": "i64"
        },
        {
          "name": "feePercent",
          "type": "u64"
        },
        {
          "name": "feeRecipient",
          "type": "pubkey"
        },
        {
          "name": "initialFunds",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "market",
      "discriminator": [
        219,
        190,
        213,
        55,
        0,
        227,
        198,
        154
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "noOutcomes",
      "msg": "At least one outcome is required"
    },
    {
      "code": 6001,
      "name": "invalidB",
      "msg": "Liquidity parameter b must be greater than zero"
    },
    {
      "code": 6002,
      "name": "invalidDuration",
      "msg": "Duration must be positive"
    },
    {
      "code": 6003,
      "name": "invalidOwner",
      "msg": "Invalid owner for the mint account."
    },
    {
      "code": 6004,
      "name": "invalidMint",
      "msg": "Invalid mint account."
    },
    {
      "code": 6005,
      "name": "marketClosed",
      "msg": "Market is closed"
    },
    {
      "code": 6006,
      "name": "invalidOutcome",
      "msg": "Invalid outcome index"
    },
    {
      "code": 6007,
      "name": "invalidShares",
      "msg": "Must buy at least one share"
    },
    {
      "code": 6008,
      "name": "overflow",
      "msg": "Overflow occurred"
    },
    {
      "code": 6009,
      "name": "underflow",
      "msg": "Underflow occurred"
    },
    {
      "code": 6010,
      "name": "mathError",
      "msg": "Math error"
    },
    {
      "code": 6011,
      "name": "marketNotClosed",
      "msg": "Market not closed yet"
    },
    {
      "code": 6012,
      "name": "marketAlreadySettled",
      "msg": "Market already settled"
    },
    {
      "code": 6013,
      "name": "unauthorized",
      "msg": "unauthorized"
    },
    {
      "code": 6014,
      "name": "noFeesToWithdraw",
      "msg": "No fees to withdraw"
    },
    {
      "code": 6015,
      "name": "noSharesToClaim",
      "msg": "No shares to claim"
    },
    {
      "code": 6016,
      "name": "insufficientFunds",
      "msg": "Insufficient funds"
    },
    {
      "code": 6017,
      "name": "outcomeNameTooLong",
      "msg": "make it short assholde"
    },
    {
      "code": 6018,
      "name": "invalidMintKey",
      "msg": "This is not fucking acceptable"
    },
    {
      "code": 6019,
      "name": "insufficientShares",
      "msg": "Go, get them first"
    },
    {
      "code": 6020,
      "name": "invalidAccounts",
      "msg": "Baz aa ja tou bahi"
    },
    {
      "code": 6021,
      "name": "invalidMintAuthority",
      "msg": "Tou b madarchod"
    },
    {
      "code": 6022,
      "name": "mintAlreadyInitialized",
      "msg": "Tou b madarchod"
    }
  ],
  "types": [
    {
      "name": "market",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "marketId",
            "type": "u64"
          },
          {
            "name": "title",
            "type": "string"
          },
          {
            "name": "oracle",
            "type": "pubkey"
          },
          {
            "name": "b",
            "type": "u64"
          },
          {
            "name": "feePercent",
            "type": "u64"
          },
          {
            "name": "feeRecipient",
            "type": "pubkey"
          },
          {
            "name": "outcomes",
            "type": {
              "vec": {
                "defined": {
                  "name": "outcome"
                }
              }
            }
          },
          {
            "name": "endTimestamp",
            "type": "i64"
          },
          {
            "name": "marketClosed",
            "type": "bool"
          },
          {
            "name": "marketSettled",
            "type": "bool"
          },
          {
            "name": "winningOutcome",
            "type": "u64"
          },
          {
            "name": "marketMakerFunds",
            "type": "u64"
          },
          {
            "name": "initialFunds",
            "type": "u64"
          },
          {
            "name": "collectedFees",
            "type": "u64"
          },
          {
            "name": "baseTokenMint",
            "type": "pubkey"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "outcome",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "totalShares",
            "type": "u64"
          },
          {
            "name": "mint",
            "type": "pubkey"
          }
        ]
      }
    }
  ]
};
