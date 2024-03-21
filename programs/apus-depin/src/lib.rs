use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, TokenAccount, Token};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("12EsXujv2DgAgMBKzBJSS92tvMWQefg8C8kpfSP11o1E");

#[program]
pub mod apus_depin {
    use anchor_spl::token::{mint_to, MintTo};

    use super::*;

    pub fn registercompute(ctx:Context<RegisterCompute>, instruction_data: ComputerProviderArgs) -> Result<()> {
        let computeinfo = &mut ctx.accounts.computeprovider;
        computeinfo.id = instruction_data.id;
        computeinfo.owner = instruction_data.owner;
        computeinfo.cards.memory = instruction_data.cards.memory;
        computeinfo.cards.model = instruction_data.cards.model;
        computeinfo.cuda_version = instruction_data.cuda_version;
        computeinfo.price = instruction_data.price;
        computeinfo.endpoint = instruction_data.endpoint;
        computeinfo.tasks = Vec::new();

        Ok(())
    }

    pub fn registeragent(ctx:Context<RegisterAgent>, instruction_data: AgentArgs) -> Result<()> {
        let agentinfo = &mut ctx.accounts.agent;
        agentinfo.owner = instruction_data.owner;
        agentinfo.post = instruction_data.post;
        agentinfo.title = instruction_data.title;
        agentinfo.description = instruction_data.description;
        agentinfo.model_hash = instruction_data.model_hash;
        agentinfo.model_type = instruction_data.model_type;
        agentinfo.docker_image_href = instruction_data.docker_image_href;
        agentinfo.api_type = instruction_data.api_type;
        agentinfo.api_doc = instruction_data.api_doc;
        agentinfo.api_default_port = instruction_data.api_default_port;
        agentinfo.price = instruction_data.price;

        Ok(())
    }

    pub fn bind_agents(ctx: Context<UpdateCompute>, instruction_data: String) -> Result<()>{
        let computeinfo = &mut ctx.accounts.computeprovider;
        computeinfo.tasks.push(instruction_data);

        Ok(())
    }

    // pub fn registeraitask(ctx: Context<RegisterAiTask>, instruction_data: AiTaskArgs) -> Result<()> {
    //     let aitaskinfo = &mut ctx.accounts.aitask;
    //     aitaskinfo.user = instruction_data.user;
    //     aitaskinfo.node = instruction_data.node;
    //     aitaskinfo.agent_hash = instruction_data.agent_hash;
    //     aitaskinfo.user_sig = instruction_data.user_sig;
    //     aitaskinfo.user_limit = instruction_data.user_limit;
    //     aitaskinfo.user_timestamp = instruction_data.user_timestamp;
    //     aitaskinfo.proof_of_work = instruction_data.proof_of_work;
    //     aitaskinfo.node_signature = instruction_data.node_signature;
    //     aitaskinfo.node_timestamp = instruction_data.node_timestamp;
    //     aitaskinfo.price = instruction_data.price;

    //     Ok(())
    // }

    pub fn submit_task(ctx: Context<RegisterAiTask>, user_balance: u32, agent_bonus: u64) -> Result<()> {
        if ctx.accounts.user_sig.is_signer &&
            ctx.accounts.node_signature.is_signer &&
            user_balance >= ctx.accounts.aitask.price {
            //扣用户的user_balance
            //调用铸币厂给agent和gpu node owner
            
            let mint_bump: u8 = ctx.bumps.mint;
            let seeds: &[&[&[u8]]; 1] = &[&["mint".as_bytes(), &[mint_bump]]];

            mint_to(
                CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.token_program.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info()
                },
                seeds,
                ),
                agent_bonus
            )?;

            
            mint_to(
                
                CpiContext::new_with_signer(

                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.token_program.to_account_info(),
                    to: ctx.accounts.token_account2.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info()
                },
                seeds,
                ),
                100-agent_bonus
            )?;
        }

        Ok(())
    }

    static mut TaskStore: Vec<StoreStruct> = Vec::new();
    pub fn batch_submit_task(ctx: Context<RegisterAiTask>, user_balance: u32, agent_bonus: u64) -> Result<()> {
        let a = StoreStruct{
            ctx,
            user_balance,
            agent_bonus
        };

        unsafe{
            TaskStore.push(a);

            if TaskStore.len() == 5 {
                for task in TaskStore.drain(..){
                    submit_task(task.ctx, task.user_balance, task.agent_bonus)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(instruction_data: ComputerProviderArgs)]
pub struct RegisterCompute<'info> {
    #[account(
        init,
        payer = compute,
        space = 8 + 32 + 20,
    )]
    pub computeprovider: Account<'info, ComputerProvider>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub compute: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(instruction_data: String)]
pub struct UpdateCompute<'info> {
    #[account(
        mut,
        realloc = 8 + 32 + 4 + instruction_data.len(),
        realloc::zero = true,
        realloc::payer = compute,
    )]
    pub computeprovider: Account<'info, ComputerProvider>,

    #[account(mut)]
    pub compute: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(instruction_data: AgentArgs, user_balance: u32, agent_bonus:u64)]
pub struct RegisterAgent<'info> {
    #[account(init, payer = agent, space = 1024)]
    pub agent: Account<'info, Agent>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub _agent: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
}

//还需要一个用户的签名 这样才能把他的钱转走
#[derive(Accounts)]
#[instruction(instruction_data: AiTaskArgs, user_balance: u32, agent_bonus: u64)]
pub struct RegisterAiTask<'info> {
    #[account(
        init,
        payer = paypay,
        space = 1024,
    )]
    pub aitask: Account<'info, AiTask>,
    pub agent: Account<'info, Agent>,
    pub compute: Account<'info, ComputerProvider>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub paypay: Signer<'info>,

    #[account(mut)]
    pub user_sig: Signer<'info>,

    #[account(mut)]
    pub node_signature: Signer<'info>,

    #[account(
        seeds = ["mint".as_bytes()],
        bump,
        mut
    )]
    pub mint: Account<'info, Mint>,
    // reward agent
    #[account(
        init_if_needed,
        payer = paypay,
        associated_token::mint = mint,
        associated_token::authority = agent,
    )]
    pub token_account: Account<'info, TokenAccount>,//应该有两个人的tokenaccount
    //reward compute
    #[account(
        init_if_needed,
        payer = paypay,
        associated_token::mint = mint,
        associated_token::authority = compute,
    )]
    pub token_account2: Account<'info, TokenAccount>,

    
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[account]
pub struct Cards {
    pub model: String,
    pub memory: String,
}

#[account]
pub struct ComputerProvider {
    pub id: String,
    pub owner: String,
    pub cards: Cards,
    pub cuda_version: String,
    pub price: String,
    pub endpoint: String,
    pub tasks: Vec<String>
}

#[account]
pub struct Agent {
    pub owner: String,
    pub post: String,
    pub title: String,
    pub description: String,
    pub model_hash: String,
    pub model_type: String,
    pub docker_image_href: String,
    pub api_type: String,
    pub api_doc: String,
    pub api_default_port: String,
    pub price: String,
}


const AGENT_VEC: Vec<Agent> = Vec::new();

const COMPUTERPROVIDER: Vec<ComputerProvider> = Vec::new();

#[account]
pub struct AgentsCompute{
    pub compute_provider_id: String,
    pub agent_hash: String,
    pub price: String
}

#[account]
pub struct AiTask{
    pub user: Pubkey,
    pub node: Pubkey,
    pub agent_hash: String,
    pub user_sig: [u8; 64],
    pub user_limit: String,
    pub user_timestamp: String,
    pub proof_of_work: String,
    pub node_signature: [u8; 64],
    pub node_timestamp: String,
    pub price: u32
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct CardsArgs {
    pub model: String,
    pub memory: String,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct ComputerProviderArgs {
    pub id: String,
    pub owner: String,
    pub cards: CardsArgs,
    pub cuda_version: String,
    pub price: String,
    pub endpoint: String,
    pub tasks: [String; 1000]
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct AgentArgs {
    pub owner: String,
    pub post: String,
    pub title: String,
    pub description: String,
    pub model_hash: String,
    pub model_type: String,
    pub docker_image_href: String,
    pub api_type: String,
    pub api_doc: String,
    pub api_default_port: String,
    pub price: String,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct AiTaskArgs{
    pub user: Pubkey,
    pub node: Pubkey,
    pub user_sig: [u8; 64],
    pub agent_hash: String,
    pub user_limit: String,
    pub user_timestamp: String,
    pub proof_of_work: String,
    pub node_timestamp: String,
    pub node_signature: [u8; 64],
    pub price: u32
}

pub struct StoreStruct{
    ctx: Context<RegisterAiTask>,
    pub user_balance: u32,
    pub agent_bonus: u64
}