#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

/// Hardcoded 50% discount codes for friends and partners (each usable once).
const DISCOUNT_CODES: &[&[u8]] = &[b"FRIEND50", b"PARTNER50", b"NEVERSEA50"];

/// NeverSea Festival 2025 registration contract.
#[multiversx_sc::contract]
pub trait MyNeversea2025 {
    #[init]
    fn init(&self, registration_fee: BigUint, registration_fee_vip: BigUint) {
        self.set_fees(registration_fee, registration_fee_vip);
    }

    #[upgrade]
    fn upgrade(&self) {}

    // ---------------- endpoints ----------------

    /// Pay `registration_fee` -> regular participant,
    /// pay `registration_fee_vip` -> VIP participant, anything else is rejected.
    #[payable("EGLD")]
    #[endpoint]
    fn register(&self) {
        let payment = self.call_value().egld().clone_value();
        let fee = self.registration_fee().get();
        let fee_vip = self.registration_fee_vip().get();
        self.register_caller(&payment, &fee, &fee_vip);
    }

    /// Register with a 50% discount code (regular or VIP, depending on the amount paid).
    #[payable("EGLD")]
    #[endpoint(registerWithDiscount)]
    fn register_with_discount(&self, code: ManagedBuffer) {
        require!(self.is_valid_code(&code), "Invalid discount code");
        require!(
            !self.used_discount_codes().contains(&code),
            "Discount code already used"
        );

        let payment = self.call_value().egld().clone_value();
        let fee = self.registration_fee().get() / 2u32;
        let fee_vip = self.registration_fee_vip().get() / 2u32;
        self.register_caller(&payment, &fee, &fee_vip);

        self.used_discount_codes().insert(code);
    }

    /// Owner updates only the regular registration fee.
    #[only_owner]
    #[endpoint(updateRegistrationFees)]
    fn update_registration_fees(&self, new_fee: BigUint) {
        require!(new_fee > 0u32, "Fee must be positive");
        self.registration_fee().set(new_fee);
    }

    /// Owner updates both fees (e.g. switch from Early Bird to Full price).
    #[only_owner]
    #[endpoint(updateAllFees)]
    fn update_all_fees(&self, registration_fee: BigUint, registration_fee_vip: BigUint) {
        self.set_fees(registration_fee, registration_fee_vip);
    }

    // ---------------- helpers ----------------

    fn set_fees(&self, registration_fee: BigUint, registration_fee_vip: BigUint) {
        require!(registration_fee > 0u32, "Fee must be positive");
        require!(
            registration_fee_vip > registration_fee,
            "VIP fee must be greater than the regular fee"
        );
        self.registration_fee().set(registration_fee);
        self.registration_fee_vip().set(registration_fee_vip);
    }

    fn register_caller(&self, payment: &BigUint, fee: &BigUint, fee_vip: &BigUint) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.participants().contains(&caller) && !self.vip_participants().contains(&caller),
            "Already registered"
        );

        if payment == fee_vip {
            self.vip_participants().insert(caller);
        } else if payment == fee {
            self.participants().insert(caller);
        } else {
            sc_panic!("Wrong payment amount, registration denied");
        }
    }

    fn is_valid_code(&self, code: &ManagedBuffer) -> bool {
        DISCOUNT_CODES.iter().any(|c| code == &ManagedBuffer::new_from_bytes(c))
    }

    // ---------------- storage ----------------

    #[view(getParticipants)]
    #[storage_mapper("participants")]
    fn participants(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getVipParticipants)]
    #[storage_mapper("vip_participants")]
    fn vip_participants(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getRegistrationFee)]
    #[storage_mapper("registration_fee")]
    fn registration_fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getRegistrationFeeVip)]
    #[storage_mapper("registration_fee_vip")]
    fn registration_fee_vip(&self) -> SingleValueMapper<BigUint>;

    #[view(getUsedDiscountCodes)]
    #[storage_mapper("used_discount_codes")]
    fn used_discount_codes(&self) -> UnorderedSetMapper<ManagedBuffer>;
}
