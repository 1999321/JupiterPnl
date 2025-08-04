use anchor_lang::{prelude::*, Discriminator};
use solana_transaction_status::option_serializer::OptionSerializer;
// use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{bs58, pubkey::Pubkey};
// use anchor_lang::prelude::borsh::BorshDeserialize;
// use anchor_lang::prelude::borsh::BorshSerialize;
// use anchor_lang::prelude::borsh::BorshSchema;

// #[derive(BorshSerialize, BorshDeserialize, Debug, Clone, BorshSchema)]

const DISCRIMINATOR_SIZE: usize = 8;


#[event]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapInstruction {
    pub amm: Pubkey,//32
    pub input_mint: Pubkey,//32
    pub input_amount: u64,//8
    pub output_mint: Pubkey,//32
    pub output_amount: u64,//8
}

pub fn parse(
    meta: &solana_transaction_status::UiTransactionStatusMeta,
) -> Vec<SwapInstruction> {
    let mut swap_instructions: Vec<SwapInstruction> = vec![];
    match meta.inner_instructions.as_ref() {
        OptionSerializer::Some(inner_instructions) => {
            for instruction in inner_instructions {
                for inner_instruction in &instruction.instructions {
                    match inner_instruction {
                        solana_transaction_status::UiInstruction::Parsed(instruction_parsed) => {
                            match instruction_parsed {
                                solana_transaction_status::UiParsedInstruction::Parsed(parsed_instruction) => {
                                    // 处理解析后的指令
                                    if parsed_instruction.program == "spl-associated-token-account" || parsed_instruction.program == "spl-token" || parsed_instruction.program_id == "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" {
                                        
                                    }
                                },
                                // 用于描述那些无法被 RPC 节点完全解析的指令，但提供了指令的基本信息和原始字节数据
                                solana_transaction_status::UiParsedInstruction::PartiallyDecoded(parsed_instruction) => {
                                    // 处理解析后的指令
                                    if parsed_instruction.program_id == "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4" {
                                        //let data = hex::decode("e445a52e51cb9a1d40c6cde8260871e2a5d5ca9e04cf5db590b714ba2fe32cb159133fc1c192b72257fd07d39cb0401e069b8857feab8184fb687f634618c035dac439dc1aeb3b5598a0f00000000001602c6103000000001d8ccf87ac0147bae756eb963a2ef6244c9691569a8ec08f0020a2eb8fbdb5a121c88d1000000000");
                                        let data = bs58::decode(&parsed_instruction.data)
                                            .into_vec();
                                        if data.is_ok() {
                                            let data = data.unwrap();
                                            let (_, buffer) = data.split_at(DISCRIMINATOR_SIZE);
                                            // println!("buffer: {:?} {:?}", buffer.len(), SwapInstruction::try_from_slice(buffer.split_at(8).1));
                                            // println!("Discriminator: {:?}", discriminator);
                                            // println!("Discriminator as hex: {:?}", SwapInstruction::DISCRIMINATOR);
                                            // let res = SwapInstruction::DISCRIMINATOR.eq(discriminator).then(
                                            //     || SwapInstruction::try_from_slice(buffer)
                                            // );
                                            let res = SwapInstruction::try_from_slice(buffer.split_at(8).1);
                                            match res {
                                                Ok(swap_instruction) => {

                                                    #[cfg(test)]
                                                    println!("Parsed Jupiter V6 Swap Instruction: {:?}", swap_instruction);

                                                    swap_instructions.push(swap_instruction);
                                                },
                                                _ => {
                                                    println!("Failed to parse Jupiter V6 Swap Instruction: {:?}", data);
                                                }
                                            }
                                            // let swap_ix = anchor_lang::prelude::borsh::try_from_slice_with_schema::<SwapInstruction>(&data.unwrap());
                                            // //let swap_ix = solana_sdk::borsh0_10::try_from_slice_unchecked::<SwapInstruction>(&data.unwrap());
                                            // println!("Parsed Jupiter V6 Swap Instruction: {:?}", swap_ix);
                                            // match swap_ix {
                                            //     Ok(swap_instruction) => {
                                            //         swap_instructions.push(
                                            //             swap_instruction
                                            //         )
                                            //     },
                                            //     Err(_) => {
                                                    
                                            //     }
                                            // }
                                        }
                                    }
                                }
                            }
                        },
                        _ => {
                            
                        }
                    }
                }
            }
        },
        OptionSerializer::None => {
            #[cfg(test)]
            println!("No inner instructions found.");
        },
        OptionSerializer::Skip => {
            #[cfg(test)]
            println!("Skipping inner instructions.");
        }
    }

    swap_instructions
}