// 导入所需的库和模块
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
};

// 定义一个结构体来存储人工智能模型数据
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AIModel {
    pub is_initialized: bool,
    pub name: String,
    pub description: String,
    pub owner: Pubkey,
    pub price: u64,
    pub model_file: Vec<u8>,
}

// 实现IsInitialized trait来检查AIModel是否已初始化
impl IsInitialized for AIModel {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

// 实现Pack trait来序列化和反序列化AIModel
impl Pack for AIModel {
    const LEN: usize = 1 + 32 + 32 + 8 + 8 + 1024;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let mut offset = 0;
        output[offset] = self.is_initialized as u8;
        offset += 1;
        output[offset..offset+32].copy_from_slice(self.name.as_bytes());
        offset += 32;
        output[offset..offset+32].copy_from_slice(self.description.as_bytes());
        offset += 32;
        output[offset..offset+8].copy_from_slice(&self.owner.to_bytes());
        offset += 8;
        output[offset..offset+8].copy_from_slice(&self.price.to_le_bytes());
        offset += 8;
        output[offset..offset+1024].copy_from_slice(&self.model_file);
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;
        let is_initialized = match input.get(offset) {
            Some(val) => *val != 0,
            None => return Err(ProgramError::InvalidAccountData),
        };
        offset += 1;
        let name = match String::from_utf8(input[offset..offset+32].to_vec()) {
            Ok(val) => val,
            Err(_) => return Err(ProgramError::InvalidAccountData),
        };
        offset += 32;
        let description = match String::from_utf8(input[offset..offset+32].to_vec()) {
            Ok(val) => val,
            Err(_) => return Err(ProgramError::InvalidAccountData),
        };
        offset += 32;
        let owner = match Pubkey::new_from_array(input[offset..offset+32].try_into().unwrap()) {
            Ok(val) => val,
            Err(_) => return Err(ProgramError::InvalidAccountData),
        };
        offset += 32;
        let price = u64::from_le_bytes(input[offset..offset+8].try_into().unwrap());
        offset += 8;
        let model_file = input[offset..offset+1024].to_vec();
        Ok(Self {
            is_initialized,
            name,
            description,
            owner,
            price,
            model_file,
        })
    }
}

// 定义一个处理程序函数来创建新的AIModel
pub fn create_ai_model(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    description: String,
    price: u64,
    model_file: Vec<u8>,
) -> ProgramResult {
    // 获取账户信息和系统变量
    let account_info_iter = &mut accounts.iter();
    let ai_model_account = next_account_info(account_info_iter)?;
    let owner_account = next_account_info(account_info_iter)?;
    let rent_sysvar_account = next_account_info(account_info_iter)?;

    // 检查AIModel账户是否已初始化
    if ai_model_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    if ai_model_account.data_len() != AIModel::LEN {
        return Err(ProgramError::InvalidAccountDataSize);
    }
    if !ai_model_account.is_uninitialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // 检查所有者账户是否具有足够的余额来支付租金
    let rent = &Rent::from_account_info(rent_sysvar_account)?;
    if !rent.is_exempt(ai_model_account.lamports(), ai_model_account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }
    if owner_account.lamports() < rent.minimum_balance(ai_model_account.data_len()) {
        return Err(ProgramError::InsufficientFunds);
    }

    // 初始化AIModel账户并存储数据
    let mut ai_model_data = AIModel::default();
    ai_model_data.is_initialized = true;
    ai_model_data.name = name;
    ai_model_data.description = description;
    ai_model_data.owner = *owner_account.key;
    ai_model_data.price = price;
    ai_model_data.model_file = model_file;
    ai_model_data.pack_into_slice(&mut ai_model_account.data.borrow_mut());

    // 转移所有者账户的余额以支付租金
    **owner_account.lamports.borrow_mut() -= rent.minimum_balance(ai_model_account.data_len());
    **ai_model_account.lamports.borrow_mut() += rent.minimum_balance(ai_model_account.data_len());

    Ok(())
}

// 入口点函数
entrypoint!(process_instruction);

// 编写测试用例
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_ai_model() {
        // 编写测试逻辑
        // ...
    }
}
