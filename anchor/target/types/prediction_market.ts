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
