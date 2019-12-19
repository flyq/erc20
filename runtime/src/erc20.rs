/// A runtime module template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue, StorageMap, Parameter, ensure};
use system::{self, ensure_signed};
use rstd::prelude::*;
use parity_codec::Codec;
use runtime_primitives::traits::{CheckedSub, CheckedAdd, Member, SimpleArithmetic, As};

// The module's configuration trait.
pub trait Trait: system::Trait {
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + As<usize> + As<u128>;    
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as erc20 {
        // bool flag to allow init to be called only once
        Init get(is_init): bool;

        // owner gets all the tokens when calls initialize
        // setting via genesis config to avoid race condition
        Owner get(owner) config(): T::AccountId;

        // total supply of the token
        // set in the genesis config
        // see ../../src/chain_spec.rs - line 105
        TotalSupply get(total_supply) config(): T::TokenBalance;

        // not really needed - name and ticker, but why not?
        Name get(name) config(): Vec<u8>;
        Ticker get (ticker) config(): Vec<u8>;

        // standard balances and allowances mappings for ERC20 implementation
        BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
        Allowance get(allowance): map (T::AccountId, T::AccountId) => T::TokenBalance;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
    fn deposit_event<T>() = default;

      // initialize the token
      // transfers the total_supply amout to the caller
      // not part of ERC20 standard interface
      // replicates the ERC20 smart contract constructor functionality
      fn init(origin) -> Result {
          let sender = ensure_signed(origin)?;
          ensure!(Self::is_init() == false, "Already initialized.");
          ensure!(Self::owner() == sender, "Only owner can initialize.");

          <BalanceOf<T>>::insert(sender.clone(), Self::total_supply());
          <Init<T>>::put(true);

          Ok(())
      }

      // transfer tokens from one account to another
      fn transfer(_origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
          let sender = ensure_signed(_origin)?;
          Self::_transfer(sender, to, value)
      }

      // approve token transfer from one account to another
      // once this is done, then transfer_from can be called with corresponding values
      fn approve(origin, spender: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
          let sender = ensure_signed(origin)?;
          // make sure the approver/owner owns this token
          ensure!(<BalanceOf<T>>::exists(&sender), "Account does not own this token");

          // get the current value of the allowance for this sender and spender combination
          // if doesnt exist then default 0 will be returned
          let allowance = Self::allowance((sender.clone(), spender.clone()));

          // add the value to the current allowance
          // using checked_add (safe math) to avoid overflow
          let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;

          // insert the new allownace value of this sender and spender combination
          <Allowance<T>>::insert((sender.clone(), spender.clone()), updated_allowance);

          // raise the approval event
          Self::deposit_event(RawEvent::Approval(sender, spender, value));
          Ok(())
      }

      // if approved, transfer from an account to another account without needing owner's signature
      fn transfer_from(_origin, from: T::AccountId, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
          let sender = ensure_signed(_origin)?;
          ensure!(<Allowance<T>>::exists((from.clone(), sender.clone())), "Allowance does not exist.");
          let allowance = Self::allowance((from.clone(), sender.clone()));
          ensure!(allowance >= value, "Not enough allowance.");

          // using checked_sub (safe math) to avoid overflow
          let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
          // insert the new allownace value of this sender and spender combination
          <Allowance<T>>::insert((from.clone(), sender.clone()), updated_allowance);

          Self::deposit_event(RawEvent::Approval(from.clone(), sender.clone(), value));
          Self::_transfer(from, to, value)
      }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
	Balance = <T as self::Trait>::TokenBalance,
    {
        // Just a dummy event.
        // Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        // To emit this event, we call the deposit funtion, from our runtime funtions
	// SomethingStored(u32, AccountId),
	
        // event for transfer of tokens
        // from, to, value
        Transfer(AccountId, AccountId, Balance),
        // event when an approval is made
        // owner, spender, value
        Approval(AccountId, AccountId, Balance),
    }
);

// module implementation block
// utility and private functions
// if marked public, accessible by other modules
impl<T: Trait> Module<T> {
    // internal transfer function for ERC20 interface
    fn _transfer(
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(<BalanceOf<T>>::exists(from.clone()), "Account does not own this token");
        let sender_balance = Self::balance_of(from.clone());
        ensure!(sender_balance >= value, "Not enough balance.");

        let updated_from_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of(to.clone());
        let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;
        
        // reduce sender's balance
        <BalanceOf<T>>::insert(from.clone(), updated_from_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert(to.clone(), updated_to_balance);

        Self::deposit_event(RawEvent::Transfer(from, to, value));
        Ok(())
    }
}


/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {
        type Event = ();
    }
    type erc20 = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(erc20::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(erc20::something(), Some(42));
        });
    }
}
