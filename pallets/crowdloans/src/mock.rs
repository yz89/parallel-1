use frame_support::{
    construct_runtime,
    dispatch::Weight,
    parameter_types, sp_io,
    traits::{
        tokens::BalanceConversion, EnsureOneOf, Everything, GenesisBuild, Nothing, SortedMembers,
    },
    weights::constants::WEIGHT_PER_SECOND,
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use orml_xcm_support::IsNativeConcrete;
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use polkadot_runtime_parachains::configuration::HostConfiguration;
use primitives::{currency::MultiCurrencyAdapter, tokens::*, Balance, ParaId};
use sp_core::H256;
use sp_runtime::{
    generic,
    traits::{
        AccountIdConversion, AccountIdLookup, BlakeTwo256, BlockNumberProvider, Convert, Zero,
    },
    AccountId32, DispatchError,
    MultiAddress::Id,
};
pub use xcm::latest::prelude::*;
pub use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
    ChildParachainAsNative, ChildParachainConvertsVia, ChildSystemParachainAsSuperuser,
    CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
    IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser, ParentIsDefault,
    RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
    SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{Config, XcmExecutor};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub type AccountId = AccountId32;
pub type CurrencyId = u32;
pub use kusama_runtime;
use primitives::ump::{XcmCall, XcmWeightFeeMisc};

pub struct RelayChainBlockNumberProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: cumulus_pallet_parachain_system::Config> BlockNumberProvider
    for RelayChainBlockNumberProvider<T>
{
    type BlockNumber = primitives::BlockNumber;

    fn current_block_number() -> Self::BlockNumber {
        cumulus_pallet_parachain_system::Pallet::<T>::validation_data()
            .map(|d| d.relay_parent_number)
            .unwrap_or_default()
            .into()
    }
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
    pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

impl cumulus_pallet_parachain_system::Config for Test {
    type Event = Event;
    type OnSystemEvent = ();
    type SelfParaId = ParachainInfo;
    type DmpMessageHandler = DmpQueue;
    type ReservedDmpWeight = ReservedDmpWeight;
    type OutboundXcmpMessageSource = XcmpQueue;
    type XcmpMessageHandler = XcmpQueue;
    type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Test {}

parameter_types! {
    pub RelayNetwork: NetworkId = NetworkId::Kusama;
    pub RelayCurrency: CurrencyId = DOT;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type XcmOriginToCallOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    XcmPassthrough<Origin>,
);

parameter_types! {
    pub const UnitWeightCost: Weight = 1;
    pub DotPerSecond: (AssetId, u128) = (AssetId::Concrete(MultiLocation::parent()), 1);
}

parameter_types! {
    pub const NativeCurrencyId: CurrencyId = HKO;
    pub GiftAccount: AccountId = PalletId(*b"par/gift").into_account();
}

pub struct GiftConvert;
impl BalanceConversion<Balance, CurrencyId, Balance> for GiftConvert {
    type Error = DispatchError;
    fn to_asset_balance(_balance: Balance, _asset_id: CurrencyId) -> Result<Balance, Self::Error> {
        Ok(Zero::zero())
    }
}

pub type LocalAssetTransactor = MultiCurrencyAdapter<
    Assets,
    IsNativeConcrete<CurrencyId, CurrencyIdConvert>,
    AccountId,
    Balance,
    LocationToAccountId,
    CurrencyIdConvert,
    NativeCurrencyId,
    ExistentialDeposit,
    GiftAccount,
    GiftConvert,
>;

pub type XcmRouter = ParachainXcmRouter<ParachainInfo>;
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct XcmConfig;
impl Config for XcmConfig {
    type Call = Call;
    type XcmSender = XcmRouter;
    type AssetTransactor = LocalAssetTransactor;
    type OriginConverter = XcmOriginToCallOrigin;
    type IsReserve = NativeAsset;
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<Ancestry>;
    type Barrier = Barrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type Trader = FixedRateOfFungible<DotPerSecond, ()>;
    type ResponseHandler = ();
    type SubscriptionService = PolkadotXcm;
    type AssetTrap = PolkadotXcm;
    type AssetClaims = PolkadotXcm;
}

type KusamaXcmOriginToCallOrigin = (
    // A `Signed` origin of the sovereign account that the original location controls.
    SovereignSignedViaLocation<
        kusama_runtime::xcm_config::SovereignAccountOf,
        kusama_runtime::Origin,
    >,
    // A child parachain, natively expressed, has the `Parachain` origin.
    ChildParachainAsNative<polkadot_runtime_parachains::origin::Origin, kusama_runtime::Origin>,
    // The AccountId32 location type can be expressed natively as a `Signed` origin.
    SignedAccountId32AsNative<kusama_runtime::xcm_config::KusamaNetwork, kusama_runtime::Origin>,
    // A system child parachain, expressed as a Superuser, converts to the `Root` origin.
    ChildSystemParachainAsSuperuser<ParaId, kusama_runtime::Origin>,
);

pub type KusamaCall = kusama_runtime::Call;
pub type KusamaLocalAssetTransactor = kusama_runtime::xcm_config::LocalAssetTransactor;
// pub type KusamaXcmOriginToCallOrigin = kusama_runtime::LocalOriginConverter;
// pub type KusamaLocationInverter = kusama_runtime::LocationInverter;
pub type KusamaAncestry = kusama_runtime::xcm_config::Ancestry;
pub type KusamaBarrier = kusama_runtime::xcm_config::Barrier;
pub type KusamaXcmPallet = kusama_runtime::XcmPallet;

pub struct RelayXcmConfig;
impl Config for RelayXcmConfig {
    type Call = KusamaCall;
    type XcmSender = RelayChainXcmRouter;
    type AssetTransactor = KusamaLocalAssetTransactor;
    type OriginConverter = KusamaXcmOriginToCallOrigin;
    type IsReserve = ();
    type IsTeleporter = ();
    type LocationInverter = LocationInverter<KusamaAncestry>;
    type Barrier = KusamaBarrier;
    type Weigher = FixedWeightBounds<UnitWeightCost, KusamaCall, MaxInstructions>;
    type Trader = FixedRateOfFungible<DotPerSecond, ()>;
    type ResponseHandler = KusamaXcmPallet;
    type SubscriptionService = KusamaXcmPallet;
    type AssetTrap = KusamaXcmPallet;
    type AssetClaims = KusamaXcmPallet;
}

impl cumulus_pallet_xcmp_queue::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = ();
}

impl cumulus_pallet_dmp_queue::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Test {
    type Event = Event;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Test {
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

    type Origin = Origin;
    type Call = Call;
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;

    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

pub struct CurrencyIdConvert;
impl Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdConvert {
    fn convert(id: CurrencyId) -> Option<MultiLocation> {
        match id {
            DOT => Some(MultiLocation::parent()),
            XDOT => Some(MultiLocation::new(
                1,
                X2(
                    Parachain(ParachainInfo::parachain_id().into()),
                    GeneralKey(b"xDOT".to_vec()),
                ),
            )),
            _ => None,
        }
    }
}

impl Convert<MultiLocation, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(location: MultiLocation) -> Option<CurrencyId> {
        match location {
            MultiLocation {
                parents: 1,
                interior: Here,
            } => Some(DOT),
            MultiLocation {
                parents: 1,
                interior: X2(Parachain(id), GeneralKey(key)),
            } if ParaId::from(id) == ParachainInfo::parachain_id() && key == b"xDOT".to_vec() => {
                Some(XDOT)
            }
            _ => None,
        }
    }
}

impl Convert<MultiAsset, Option<CurrencyId>> for CurrencyIdConvert {
    fn convert(a: MultiAsset) -> Option<CurrencyId> {
        if let MultiAsset {
            id: AssetId::Concrete(id),
            fun: _,
        } = a
        {
            Self::convert(id)
        } else {
            None
        }
    }
}

pub struct AccountIdToMultiLocation;
impl Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
    fn convert(account_id: AccountId) -> MultiLocation {
        MultiLocation::from(Junction::AccountId32 {
            network: NetworkId::Any,
            id: account_id.into(),
        })
    }
}

parameter_types! {
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));
    pub const BaseXcmWeight: Weight = 100_000_000;
    pub const MaxInstructions: u32 = 100;
    pub const MaxAssetsForTransfer: usize = 2;
}

impl orml_xtokens::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type CurrencyIdConvert = CurrencyIdConvert;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type SelfLocation = SelfLocation;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type BaseXcmWeight = BaseXcmWeight;
    type LocationInverter = LocationInverter<Ancestry>;
    type MaxAssetsForTransfer = MaxAssetsForTransfer;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type BlockNumber = u32;
type Index = u32;
pub const DOT_DECIMAL: u128 = 10u128.pow(10);

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Origin = Origin;
    type Call = Call;
    type Index = Index;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

pub struct AliceOrigin;
impl SortedMembers<AccountId> for AliceOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![ALICE]
    }
}

pub struct BobOrigin;
impl SortedMembers<AccountId> for BobOrigin {
    fn sorted_members() -> Vec<AccountId> {
        vec![BOB]
    }
}

parameter_types! {
    pub const CrowdloansPalletId: PalletId = PalletId(*b"crwloans");
    pub const MinContribution: Balance = 0;
    pub const MigrateKeysLimit: u32 = 10;
    pub const RemoveKeysLimit: u32 = 1000;
    pub SelfParaId: ParaId = para_a_id();
    pub RefundLocation: AccountId = para_a_id().into_account();
}

pub type CreateVaultOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type DissolveVaultOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type RefundOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type UpdateVaultOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type VrfOrigin = EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type OpenCloseOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<AliceOrigin, AccountId>>;

pub type AuctionFailedOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<BobOrigin, AccountId>>;

pub type SlotExpiredOrigin =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureSignedBy<BobOrigin, AccountId>>;

impl crate::Config for Test {
    type Event = Event;
    type Origin = Origin;
    type Call = Call;
    type PalletId = CrowdloansPalletId;
    type SelfParaId = SelfParaId;
    type Assets = Assets;
    type RelayCurrency = RelayCurrency;
    type MinContribution = MinContribution;
    type MigrateKeysLimit = MigrateKeysLimit;
    type RemoveKeysLimit = RemoveKeysLimit;
    type MigrateOrigin = EnsureRoot<AccountId>;
    type CreateVaultOrigin = CreateVaultOrigin;
    type DissolveVaultOrigin = DissolveVaultOrigin;
    type RefundOrigin = RefundOrigin;
    type UpdateVaultOrigin = UpdateVaultOrigin;
    type VrfOrigin = VrfOrigin;
    type OpenCloseOrigin = OpenCloseOrigin;
    type AuctionFailedOrigin = AuctionFailedOrigin;
    type SlotExpiredOrigin = SlotExpiredOrigin;
    type WeightInfo = ();
    type XCM = XcmHelper;
    type RelayChainBlockNumberProvider = RelayChainBlockNumberProvider<Test>;
}

parameter_types! {
    pub const XcmHelperPalletId: PalletId = PalletId(*b"par/fees");
    pub const NotifyTimeout: BlockNumber = 100;
}

impl pallet_xcm_helper::Config for Test {
    type Event = Event;
    type UpdateOrigin = EnsureRoot<AccountId>;
    type Assets = Assets;
    type XcmSender = XcmRouter;
    type PalletId = XcmHelperPalletId;
    type RelayNetwork = RelayNetwork;
    type NotifyTimeout = NotifyTimeout;
    type AccountIdToMultiLocation = AccountIdToMultiLocation;
    type RefundLocation = RefundLocation;
    type BlockNumberProvider = frame_system::Pallet<Test>;
    type WeightInfo = ();
}

parameter_types! {
    pub const AssetDeposit: Balance = DOT_DECIMAL;
    pub const ApprovalDeposit: Balance = 0;
    pub const AssetAccountDeposit: Balance= 0;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 0;
    pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Test {
    type Event = Event;
    type Balance = Balance;
    type AssetId = CurrencyId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type AssetAccountDeposit = AssetAccountDeposit;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type WeightInfo = ();
    type Extra = ();
}

construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
        Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
        Crowdloans: crate::{Pallet, Storage, Call, Event<T>},
        ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>},
        ParachainInfo: parachain_info::{Pallet, Storage, Config},
        XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
        DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
        CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
        XcmHelper: pallet_xcm_helper::{Pallet, Storage, Call, Event<T>},
        XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>},
    }
);

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let xcm_weight_fee_misc = XcmWeightFeeMisc {
        weight: 3_000_000_000,
        fee: dot(10f64),
    };

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        Assets::force_create(Origin::root(), DOT, Id(ALICE), true, 1).unwrap();
        Assets::force_create(Origin::root(), XDOT, Id(ALICE), true, 1).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, Id(ALICE), dot(100f64)).unwrap();
        Assets::mint(Origin::signed(ALICE), XDOT, Id(ALICE), dot(100f64)).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            DOT,
            Id(XcmHelper::account_id()),
            dot(30f64),
        )
        .unwrap();
        XcmHelper::update_xcm_weight_fee(Origin::root(), XcmCall::AddMemo, xcm_weight_fee_misc)
            .unwrap();
    });

    ext
}

//initial parchain and relaychain for testing
decl_test_parachain! {
    pub struct ParaA {
        Runtime = Test,
        XcmpMessageHandler = XcmpQueue,
        DmpMessageHandler = DmpQueue,
        new_ext = para_ext(1),
    }
}

decl_test_relay_chain! {
    pub struct Relay {
        Runtime = kusama_runtime::Runtime,
        XcmConfig = RelayXcmConfig,
        new_ext = relay_ext(),
    }
}

decl_test_network! {
    pub struct TestNet {
        relay_chain = Relay,
        parachains = vec![
            (1, ParaA),
        ],
    }
}

pub type KusamaRuntime = kusama_runtime::Runtime;
pub type RelayRegistrar = polkadot_runtime_common::paras_registrar::Pallet<KusamaRuntime>;
pub type RelayParas = polkadot_runtime_parachains::paras::Pallet<KusamaRuntime>;
pub type RelayCrowdloan = polkadot_runtime_common::crowdloan::Pallet<KusamaRuntime>;
pub type RelayInitializer = polkadot_runtime_parachains::initializer::Pallet<KusamaRuntime>;
pub type RelayCrowdloanEvent = polkadot_runtime_common::crowdloan::Event<KusamaRuntime>;
pub type RelaySystem = frame_system::Pallet<KusamaRuntime>;
pub type RelayEvent = kusama_runtime::Event;

pub fn para_a_id() -> ParaId {
    ParaId::from(1)
}

pub fn parathread_id() -> ParaId {
    ParaId::from(2001)
}

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let xcm_weight_fee_misc = XcmWeightFeeMisc {
        weight: 3_000_000_000,
        fee: dot(10f64),
    };

    let parachain_info_config = parachain_info::GenesisConfig {
        parachain_id: para_id.into(),
    };
    <parachain_info::GenesisConfig as GenesisBuild<Test, _>>::assimilate_storage(
        &parachain_info_config,
        &mut t,
    )
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        Assets::force_create(Origin::root(), DOT, Id(ALICE), true, 1).unwrap();
        Assets::force_create(Origin::root(), XDOT, Id(ALICE), true, 1).unwrap();
        Assets::mint(Origin::signed(ALICE), DOT, Id(ALICE), dot(100_000f64)).unwrap();
        Assets::mint(Origin::signed(ALICE), XDOT, Id(ALICE), dot(100f64)).unwrap();
        Assets::mint(
            Origin::signed(ALICE),
            DOT,
            Id(XcmHelper::account_id()),
            dot(30f64),
        )
        .unwrap();
        XcmHelper::update_xcm_weight_fee(Origin::root(), XcmCall::AddMemo, xcm_weight_fee_misc)
            .unwrap();
    });

    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use kusama_runtime::{Runtime, System};
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, dot(100_000f64)),
            (para_a_id().into_account(), dot(1_000_000f64)),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
        config: HostConfiguration {
            max_code_size: 1024u32,
            ..Default::default()
        },
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn dot(n: f64) -> Balance {
    ((n * 1000000f64) as u128) * DOT_DECIMAL / 1000000u128
}
