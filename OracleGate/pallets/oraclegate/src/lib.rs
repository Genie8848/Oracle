#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn get_item)]
    pub(super) type Items<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        CommodityId<T>,
        T::AccountId,
    >;

    #[pallet::storage]
    #[pallet::getter(fn total_nft)]
    pub type TotalNft<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn commodities_for_account)]
    /// A mapping from an account to a list
    /// of all of the commodities of this type that are owned by it.
    pub(super) type CommoditiesForAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Vec<CommodityId<T>>,
        ValueQuery
    >;

    pub type CommodityId<T> = <T as frame_system::Config>::Hash;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Burned(T::AccountId, CommodityId<T>),
        Minted(T::AccountId, CommodityId<T>),
        Transferred {
            item: CommodityId<T>,
            from: T::AccountId,
            to: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        AlreadyExists,
        DoesNotExist,
        NotTheOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Mint nft
        #[pallet::weight(0)]
        pub fn mint(origin: OriginFor<T>, item: CommodityId<T>, owner: T::AccountId) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let _who = ensure_signed(origin)?;
            // ensure commodity doesn't already exist
            ensure!(!Items::<T>::contains_key(item),Error::<T>::AlreadyExists);
            // mint nft
            TotalNft::<T>::mutate(|n| *n += 1);
            Items::<T>::insert(item.clone(), owner.clone());
            CommoditiesForAccount::<T>::mutate(&owner, |nfts| {
                nfts.push(item.clone())
            });
            Self::deposit_event(Event::Minted(owner, item));
            Ok(().into())
        }

        /// Burn nft
        #[pallet::weight(0)]
        pub fn burn(origin: OriginFor<T>, item: CommodityId<T>, owner: T::AccountId) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let _ = ensure_signed(origin)?;
            // ensure commodity exists
            ensure!(Items::<T>::contains_key(item),Error::<T>::DoesNotExist);
            // ensure this is the owner
            let nfts_owned = CommoditiesForAccount::<T>::get(&owner);
            ensure!(nfts_owned.contains(&item),Error::<T>::NotTheOwner);
            // burn nft
            TotalNft::<T>::mutate(|n| *n -= 1);
            Items::<T>::remove(item.clone());
            CommoditiesForAccount::<T>::mutate(&owner, |nfts| {
                let pos = nfts.iter().position(|i| i == &item).unwrap();
                nfts.remove(pos);
            });
            Self::deposit_event(Event::Burned(owner, item));
            Ok(().into())
        }

        /// Transfer nft
        #[pallet::weight(0)]
        pub fn transfer(origin: OriginFor<T>, item: CommodityId<T>, owner: T::AccountId, dest: T::AccountId) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            let _ = ensure_signed(origin)?;
            // ensure commodity exists
            ensure!(Items::<T>::contains_key(item),Error::<T>::DoesNotExist);
            // ensure this is the owner
            let nfts_owned = CommoditiesForAccount::<T>::get(&owner);
            ensure!(nfts_owned.contains(&item),Error::<T>::NotTheOwner);
            // transfer to new owner
            Items::<T>::insert(item.clone(), dest.clone());
            CommoditiesForAccount::<T>::mutate(&owner, |nfts| {
                let pos = nfts.iter().position(|i| i == &item).unwrap();
                nfts.remove(pos);
            });
            CommoditiesForAccount::<T>::mutate(&owner, |nfts| {
                nfts.push(item.clone())
            });
            Self::deposit_event(Event::Transferred {
                item,
                from: owner,
                to: dest,
            });
            Ok(().into())
        }
    }
}
