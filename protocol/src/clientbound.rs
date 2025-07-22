use packet::types::{VarInt, NBT, UUID};
use packet_macros::{Packet, PacketReadable, PacketWritable};
use serde::{Deserialize, Serialize};

#[derive(Packet, Debug)]
#[packet_id(0x01)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct EncryptionRequestPacket {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
    pub should_authenticate: bool,
}

#[derive(Packet, Debug)]
#[packet_id(0x02)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct LoginSuccessPacket {
    pub uuid: UUID,
    pub username: String,
    pub properties: Vec<Property>,
}

#[derive(PacketReadable, PacketWritable, Debug)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub is_signed: bool,
    pub signature: Option<String>,
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Status)]
#[packet_direction(Clientbound)]
pub struct StatusResponsePacket {
    pub json: StatusResponseJson,
}

#[derive(Debug, Serialize, Deserialize, PacketReadable, PacketWritable)]
#[packet(repr = "json")]
#[serde(rename_all = "camelCase")]
pub struct StatusResponseJson {
    version: Version,
    players: Option<Players>,
    description: Option<Description>,
    favicon: Option<String>,
    enforce_secure_chat: Option<bool>,
    prevent_chat_reports: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    name: String,
    protocol: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Players {
    max: i32,
    online: i32,
    #[serde(default)]
    sample: Vec<PlayerSample>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Description {
    Text { text: String },
    String(String),
}

#[derive(Packet, Debug)]
#[packet_id(0x03)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct SetCompressionPacket {
    #[packet(repr = "VarInt")]
    pub threshold: i32,
}

#[derive(Packet, Debug)]
#[packet_id(0x00)]
#[packet_state(Login)]
#[packet_direction(Clientbound)]
pub struct DisconnectPacket {
    pub reason: NBT,
}

pub mod configuration {
    use packet::types::{VarInt, NBT, UUID};
    use packet_macros::{Packet, PacketReadable, PacketWritable};

    #[derive(Packet, Debug)]
    #[packet_id(0x00)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct CookieRequestPacket {
        pub key: String,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x01)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct ServerPluginMessagePacket {
        pub channel: String,
        #[packet(len = "rest")]
        pub data: Vec<u8>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x02)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct DisconnectPacket {
        pub reason: NBT,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x03)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct FinishConfigurationPacket {}

    #[derive(Packet, Debug)]
    #[packet_id(0x04)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct KeepAlivePacket {
        pub id: i64,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x05)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct PingPacket {
        pub id: i32,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x06)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct ResetChatPacket {}

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct RegistryEntry {
        pub entry_id: String,
        pub data: Option<String>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x07)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct RegistryDataPacket {
        pub registry_id: String,
        pub entries: Vec<RegistryEntry>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x08)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct RemoveResourcePackPacket {
        pub resource_pack_id: Option<UUID>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x09)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct AddResourcePackPacket {
        pub resource_pack_id: UUID,
        pub url: String,
        pub hash: String,
        pub forced: bool,
        pub prompt_message: Option<String>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0A)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct StoreCookiePacket {
        pub key: String,
        pub data: Vec<u8>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0B)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct TransferPacket {
        pub host: String,
        pub port: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0C)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct FeatureFlagsPacket {
        pub flags: Vec<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Tag {
        pub registry_id: String,
        pub entries: Vec<VarInt>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0D)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct UpdateTagsPacket {
        pub tags: Vec<Tag>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Pack {
        pub namespace: String,
        pub id: String,
        pub version: String,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0E)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct KnownPacksPacket {
        pub packs: Vec<Pack>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ReportDetails {
        pub title: String,
        pub description: String,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0F)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct CustomReportDetailsPacket {
        pub reports: Vec<ReportDetails>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Link {
        pub built_in: bool,
        pub label: String,
        pub url: String,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x10)]
    #[packet_state(Configuration)]
    #[packet_direction(Clientbound)]
    pub struct ServerLinksPacket {
        pub links: Vec<Link>,
    }
}

pub mod play {
    use packet::types::{
        identifier::Identifier, position::Position, vec3::Vec3, NBTValue, VarInt, NBT, UUID,
    };
    use packet_macros::{Packet, PacketReadable, PacketWritable};

    #[derive(Packet, Debug)]
    #[packet_id(0x00)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct BundleDelimiterPacket {}

    #[derive(Packet, Debug)]
    #[packet_id(0x01)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct SpawnEntityPacket {
        pub entity_id: VarInt,
        pub entity_uuid: UUID,
        pub entity_type: VarInt,
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub pitch: u8,
        pub yaw: u8,
        pub head_yaw: u8,
        pub velocity_x: i16,
        pub velocity_y: i16,
        pub velocity_z: i16,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x02)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct EntityAnimationPacket {
        pub entity_id: VarInt,
        pub animation: u8,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Statistic {
        pub category_id: VarInt,
        pub statistic_id: VarInt,
        pub value: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x03)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct AwardStatisticsPacket {
        pub statistics: Vec<Statistic>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x04)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct AcknowledgeBlockChangePacket {
        pub sequence_id: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x06)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct BlockEntityDataPacket {
        pub location: Position,
        pub entity_type: VarInt,
        pub nbt_data: NBT,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x07)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct BlockActionPacket {
        pub location: Position,
        pub action_id: u8,
        pub action_parameter: u8,
        pub block_type: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x08)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct BlockUpdatePacket {
        pub location: Position,
        pub block_id: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum BossBarActionColor {
        Ping = 0,
        Blue = 1,
        Red = 2,
        Green = 3,
        Yellow = 4,
        Purple = 5,
        White = 6,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(index = "VarInt")]
    pub enum BossBarAction {
        Add {
            title: String,
            health: f32,
            color: BossBarActionColor,
            dividers: BossBarActionColor,
            flags: u8,
        },
        Remove {},
        UpdateHealth {
            health: f32,
        },
        UpdateTitle {
            title: String,
        },
        UpdateStyle {
            color: BossBarActionColor,
            dividers: BossBarActionColor,
        },
        UpdateFlags {
            flags: u8,
        },
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x09)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct BossBarPacket {
        pub uuid: UUID,
        pub action: BossBarAction,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0A)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChangeDifficultyPacket {
        pub difficulty: u8,
        pub locked: bool,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0B)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChunkBatchFinishedPacket {
        batch_size: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0C)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChunkBatchStartedPacket {}

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ChunkBiomeData {
        pub x: i32,
        pub z: i32,
        pub biomes: Vec<i32>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0D)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChunkBiomesPacket {
        pub chunk_biome_data: Vec<ChunkBiomeData>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0E)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct CleatTitlesPacket {
        pub reset: bool,
    }

    /* #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Node {
        pub flags: u8,
        pub children: Vec<VarInt>,
        pub redirect_node: Option<VarInt>,
        pub name: Option<String>,
        pub parser_id: Option<VarInt>,
        // Impletement parsers
        pub properties: Vec<?>,
        pub suggestion_type: Option<Identifier>
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x0F)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct CommandSuggestionsResponsePacket {
        pub nodes: Vec<Node>,
        pub root_index: VarInt
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x10)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct CommandsPacket {
        pub nodes : Vec<Node>,
        pub root_index: VarInt,
    } */

    #[derive(Packet, Debug)]
    #[packet_id(0x11)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct CloseContainerPacket {
        pub window_id: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum ItemRarity {
        Common = 0,
        Uncommon = 1,
        Rare = 2,
        Epic = 3,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ItemEnchantment {
        pub id: VarInt,
        pub level: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BlockPredicateProperty {
        pub name: String,
        pub is_exact_match: bool,
        #[packet(cond = "is_exact_match")]
        pub exact_value: Option<String>,
        #[packet(cond = "!is_exact_match")]
        pub min_value: Option<String>,
        #[packet(cond = "!is_exact_match")]
        pub max_value: Option<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BlockPredicateExactDataComponentMatcher {
        pub component_type: VarInt,
        pub value: StructuredComponent,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum BlockPredicatePartialComponentMatcherType {
        Damage = 0,
        Enchantments = 1,
        StoredEnchantments = 2,
        PotionContents = 3,
        CustomData = 4,
        Container = 5,
        BundleContents = 6,
        FireworkExplosion = 7,
        Fireworks = 8,
        WritableBookContent = 9,
        WittenBookContent = 10,
        AttributeModifiers = 11,
        Trim = 12,
        JukeboxPlayable = 13,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BlockPredicatePartialComponentMatcher {
        pub component_type: BlockPredicatePartialComponentMatcherType,
        // TODO: Should always be NBT Compound, does it even have the prefix?
        pub value: NBT,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BlockPredicate {
        pub blocks: Option<IdSet>,
        pub properties: Option<Vec<BlockPredicateProperty>>,
        pub nbt: Option<NBT>,
        pub data_components: Vec<BlockPredicateExactDataComponentMatcher>,
        pub partial_components: Vec<BlockPredicatePartialComponentMatcher>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum AttributeItemSlot {
        Any = 0,
        MainHand = 1,
        OffHand = 2,
        Hand = 3,
        Feet = 4,
        Legs = 5,
        Chest = 6,
        Head = 7,
        Armor = 8,
        Body = 9,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum AttributeOperation {
        Add = 0,
        MultiplyBase = 1,
        MultiplyTotal = 2,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct AttributeModifier {
        pub attribute_id: VarInt,
        pub modifier_id: Identifier,
        pub value: f64,
        pub operation: AttributeOperation,
        pub slot: AttributeItemSlot,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct PotionEffect {
        amplifier: VarInt,
        duration: VarInt,
        ambient: bool,
        show_particles: bool,
        show_icon: bool,
        // TODO: Documentation does not mention what exactly this is, keep it here for clarity.
        has_hidden_effect: bool,
        hidden_effect: Option<u8>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(index = "VarInt")]
    pub enum ConsumeEffect {
        // TODO: Technicall, effects is array and not prefixed array, so length needs to be known from context. Not sure from where through.
        ApplyEffects {
            effects: Vec<PotionEffect>,
            probability: f32,
        },
        RemoveEffects {
            effects: IdSet,
        },
        ClearAllEffects {},
        TeleportRandomly {
            diameter: f32,
        },
        // TODO: Implement Sound Event
        PlaySound {
            sound: VarInt,
        },
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum ConsumableAnimation {
        None = 0,
        Eat = 1,
        Drink = 2,
        Block = 3,
        Bow = 4,
        Spear = 5,
        Crossbow = 6,
        Spyglass = 7,
        TootHorn = 8,
        Brush = 9,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct IdSet {
        pub set_type: VarInt,
        #[packet(cond = "*set_type == 0")]
        pub tag_name: Option<String>,
        #[packet(cond = "*set_type != 0")]
        pub ids: Option<Vec<VarInt>>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ToolRule {
        pub blocks: IdSet,
        // Technically defined as (has_speed, speed (present if has_speed is true)), but works out the same when using prefixed Option<T> representation
        pub speed: Option<f32>,
        // Same as above
        pub correct_drop_for_block: Option<bool>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum EquipableSlot {
        MainHand = 0,
        Feet = 1,
        Legs = 2,
        Chest = 3,
        Head = 4,
        OffHand = 5,
        Body = 6,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum DyeColor {
        White = 0,
        Orange = 1,
        Magenta = 2,
        LightBlue = 3,
        Yellow = 4,
        Lime = 5,
        Pink = 6,
        Gray = 7,
        LightGray = 8,
        Cyan = 9,
        Purple = 10,
        Blue = 11,
        Brown = 12,
        Green = 13,
        Red = 14,
        Black = 15,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum FireworkShape {
        SmallBall = 0,
        LargeBall = 1,
        Star = 2,
        Creeper = 3,
        Burst = 4,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct FireworkExplosion {
        pub shape: FireworkShape,
        pub colors: Vec<i32>,
        pub fade_colors: Vec<i32>,
        pub has_trail: bool,
        pub has_twinkle: bool,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct DamageReduction {
        pub damage_types: Vec<Identifier>, // TODO: Actually should be damage type tag
        pub base: f32,
        pub factor: f32,
        pub horizontal_blocking_angle: f32,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ItemDamage {
        pub threshold: f32,
        pub base: f32,
        pub factor: f32,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct MapDecoration {
        pub icon_type: String,
        pub x: f64,
        pub z: f64,
        pub rotation: f32,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct SuspiciousStewEffect {
        pub effect_id: Identifier,
        pub duration: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct WritableBookPage {
        pub text: String,
        pub filtered: Option<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct WrittenBookPage {
        pub text: NBT,             // Text component
        pub filtered: Option<NBT>, // Text component
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct WrittenBookTitle {
        pub title: String,
        pub filtered: Option<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct LodestoneTarget {
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub dimension: Identifier,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct PlayerProfile {
        pub name: Option<String>,
        pub id: Option<UUID>,
        pub properties: Vec<ProfileProperty>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ProfileProperty {
        pub name: String,
        pub value: String,
        pub signature: Option<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BannerPattern {
        pub pattern: Identifier,
        pub color: DyeColor,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct ContainerItem {
        pub slot: VarInt,
        pub item: Slot,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct BeeData {
        pub entity_data: NBT,
        pub ticks_in_hive: VarInt,
        pub min_ticks_in_hive: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(index = "VarInt")]
    pub enum StructuredComponent {
        CustomData {
            data: NBT,
        },
        MaxStackSize {
            max_stack_size: VarInt,
        },
        MaxDamage {
            max_damage: VarInt,
        },
        Damage {
            damage: VarInt,
        },
        Unbreakable {},
        CustomName {
            name: NBT,
        },
        ItemName {
            name: String,
        },
        ItemModel {
            model: Identifier,
        },
        Lore {
            lore: Vec<NBT>,
        },
        Rarity {
            rarity: ItemRarity,
        },
        Enchantments {
            enchantments: Vec<ItemEnchantment>,
        },
        CanPlaceOn {
            block_predicates: Vec<BlockPredicate>,
        },
        CanBreak {
            block_predicates: Vec<BlockPredicate>,
        },
        AttributeModifiers {
            modifiers: Vec<AttributeModifier>,
        },
        CustomModelData {
            floats: Vec<f32>,
            flags: Vec<bool>,
            strings: Vec<String>,
            colors: Vec<i32>,
        },
        TooltipDisplay {
            hide_tooltip: bool,
            hidden_components: Vec<VarInt>,
        },
        RepairCost {
            cost: VarInt,
        },
        CreativeSlotLock {},
        EnchantmentGlintOverride {
            has_glint: bool,
        },
        IntangibleProjectile {
            empty: NBT,
        },
        Food {
            nutrition: VarInt,
            saturatio_modifier: f32,
            can_always_eat: bool,
        },
        Consumable {
            consume_seconds: f32,
            animation: ConsumableAnimation,
            sound: VarInt, // TODO: Implement ID or Sound Event
            has_consume_particles: bool,
            consume_effects: Vec<ConsumeEffect>,
        },
        UseRemainder {
            remainder: Slot,
        },
        UseCooldown {
            seconds: f32,
            cooldown_group: Option<String>, // TODO: Only present if if has cooldown group is present, implement greedy instead of prefixed as patch-fix?
        },
        DamageResistant {
            damage_types: Vec<Identifier>,
        },
        Tool {
            rules: Vec<ToolRule>,
            default_mining_speed: Option<f32>,
            damage_per_block: VarInt,
            can_destroy_blocks_in_creative: bool,
        },
        Weapon {
            damage: VarInt,
            disable_blocking_for: f32,
        },
        Enchantable {
            value: VarInt,
        },
        Equippable {
            slot: EquipableSlot,
            equip_sound: VarInt, // TODO: Implement ID or Sound Event
            model: Option<Identifier>,
            camera_overlay: Option<Identifier>,
            allowed_entities: Option<Vec<IdSet>>,
            dispensable: bool,
            swappable: bool,
            damage_on_hurt: bool,
        },
        Repairable {
            items: IdSet,
        },
        Glider {},
        TooltipStyle {
            style: Identifier,
        },
        DeathProtection {
            death_effects: Vec<ConsumeEffect>,
        },
        BlocksAttacks {
            block_delay_seconds: f32,
            disable_cooldown_scale: f32,
            damage_reductions: Vec<DamageReduction>,
            item_damage: ItemDamage,
            block_sound: VarInt,         // TODO: Implement ID or Sound Event
            disabled_sound: VarInt,      // TODO: Implement ID or Sound Event
            bypassed_by: Option<String>, // TODO: damage type tag
        },
        StoredEnchantments {
            enchantments: Vec<ItemEnchantment>,
        },
        DyedColor {
            color: i32,
        },
        MapColor {
            color: i32,
        },
        MapId {
            id: VarInt,
        },
        MapDecorations {
            decorations: Vec<MapDecoration>,
        },
        MapPostProcessing {
            post_processing: VarInt,
        },
        ChargedProjectiles {
            projectiles: Vec<Slot>,
        },
        BundleContents {
            contents: Vec<Slot>,
        },
        PotionContents {
            potion: Option<String>,
            custom_color: Option<i32>,
            custom_name: Option<String>,
            custom_effects: Vec<PotionEffect>,
        },
        PotionDurationScale {
            scale: f32,
        },
        SuspiciousStewEffects {
            effects: Vec<SuspiciousStewEffect>,
        },
        WritableBookContent {
            pages: Vec<WritableBookPage>,
        },
        WrittenBookContent {
            pages: Vec<WrittenBookPage>,
            title: WrittenBookTitle,
            author: String,
            generation: VarInt,
            resolved: bool,
        },
        Trim {
            pattern: String,  // TODO: Implement ID or Trim Material
            material: String, // TODO: Implement ID or Trim Material
        },
        DebugStickState {
            // TODO: Map of block ID to property name
            properties: NBT,
        },
        EntityData {
            entity_data: NBT,
        },
        BucketEntityData {
            entity_data: NBT,
        },
        BlockEntityData {
            block_entity_data: NBT,
        },
        Instrument {
            instrument: Identifier,
        },
        ProvidesTrimMaterial {
            trim_material: String,
        },
        OminousBottleAmplifier {
            amplifier: VarInt,
        },
        JukeboxPlayable {
            song: Identifier,
        },
        ProvidesBannerPatterns {
            banner_patterns: String, // TODO: Tag reference
        },
        Recipes {
            recipes: Vec<Identifier>,
        },
        LodestoneTracker {
            target: Option<LodestoneTarget>,
            tracked: bool,
        },
        FireworkExplosion {
            explosion: FireworkExplosion,
        },
        Fireworks {
            explosions: Vec<FireworkExplosion>,
            flight_duration: u8,
        },
        Profile {
            profile: PlayerProfile,
        },
        NoteBlockSound {
            sound: Identifier,
        },
        BannerPatterns {
            patterns: Vec<BannerPattern>,
        },
        BaseColor {
            color: DyeColor,
        },
        PotDecorations {
            decorations: Vec<Identifier>,
        },
        Container {
            items: Vec<ContainerItem>,
        },
        BlockState {
            // TODO: Map of property name to value
            properties: NBT,
        },
        Bees {
            bees: Vec<BeeData>,
        },
        Lock {
            lock_component: NBT, // TODO: Item predicate
        },
        ContainerLoot {
            loot_table: String,
            seed: i64,
        },
        BreakSound {
            sound: VarInt, // TODO: Implement ID or Sound Event
        },
        VillagerVariant {
            variant: Identifier,
        },
        WolfVariant {
            variant: Identifier,
        },
        WolfSoundVariant {
            variant: Identifier,
        },
        WolfCollar {
            color: DyeColor,
        },
        FoxVariant {
            variant: Identifier,
        },
        SalmonSize {
            size: VarInt,
        },
        ParrotVariant {
            variant: VarInt,
        },
        TropicalFishPattern {
            pattern: VarInt,
        },
        TropicalFishBaseColor {
            color: DyeColor,
        },
        TropicalFishPatternColor {
            color: DyeColor,
        },
        MooshroomVariant {
            variant: Identifier,
        },
        RabbitVariant {
            variant: VarInt,
        },
        PigVariant {
            variant: Identifier,
        },
        CowVariant {
            variant: Identifier,
        },
        ChickenVariant {
            variant: Identifier,
        },
        FrogVariant {
            variant: Identifier,
        },
        HorseVariant {
            variant: VarInt,
        },
        PaintingVariant {
            variant: Identifier,
        },
        LlamaVariant {
            variant: VarInt,
        },
        AxolotlVariant {
            variant: VarInt,
        },
        CatVariant {
            variant: Identifier,
        },
        CatCollar {
            color: DyeColor,
        },
        SheepColor {
            color: DyeColor,
        },
        ShulkerColor {
            color: DyeColor,
        },
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct Slot {
        pub item_count: VarInt,
        pub item_id: Option<VarInt>,
        pub number_of_components_to_add: Option<VarInt>,
        pub number_of_components_to_remove: Option<VarInt>,
        // TODO: Components to add (array of component type and data)
        // TODO: Components to remove (array of component type)
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x12)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct SetContainerContentPacket {
        pub window_id: VarInt,
        pub state_id: VarInt,
        pub slot_data: Vec<Slot>,
        pub carried_item: Slot,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x13)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct SetContainerPropertyPacket {
        pub window_id: VarInt,
        pub property: i16,
        pub value: i16,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x14)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct SetContainerSlotPacket {
        pub window_id: VarInt,
        pub state_id: VarInt,
        pub slot: i16,
        pub slot_data: Slot,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x15)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct CookieRequestPacket {
        pub key: String,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x16)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct SetCooldownPacket {
        pub cooldown_group: String,
        pub cooldown_ticks: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum ChatSuggestionsAction {
        Add = 0,
        Remove = 1,
        Set = 2,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x17)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChatSuggestionsPacket {
        pub action: ChatSuggestionsAction,
        pub entries: Vec<String>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x18)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ClientboundPluginMessagePacket {
        pub channel: String,
        #[packet(len = "rest")]
        pub data: Vec<u8>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x19)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct DamageEventPacket {
        pub entity_id: VarInt,
        pub source_type_id: VarInt,
        pub source_cause_id: VarInt,
        pub source_direct_id: VarInt,
        pub source_position: Option<Vec3<f64>>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum DebugSampleType {
        TickTime = 0,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1A)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct DebugSamplePacket {
        pub sample: Vec<i64>,
        pub sample_type: DebugSampleType,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1B)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct DeleteMessagePacket {
        pub message_id: VarInt,
        pub signature: Option<Vec<u8>>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1C)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct DisconnectPacket {
        pub reason: NBT,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1D)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct DisguisedChatMessagePacket {
        pub message: NBT,
        pub chat_type: VarInt, // TODO: Implement ID or Chat Type
        pub sender_name: NBT,
        pub target_name: Option<NBT>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "u8")]
    pub enum EntityStatus {
        Unknown = 0,
        // Add more entity statuses as needed
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1E)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct EntityEventPacket {
        pub entity_id: i32,
        pub entity_status: EntityStatus,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x1F)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct TeleportEntityPacket {
        pub entity_id: VarInt,
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub velocity_x: f64,
        pub velocity_y: f64,
        pub velocity_z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x20)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ExplosionPacket {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub player_delta_velocity: Option<Vec3<f64>>,
        pub explosion_particle_id: VarInt,
        pub explosion_particle_data: Vec<u8>, // TODO: Implement particle data
        pub explosion_sound: VarInt,          // TODO: Implement ID or Sound Event
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x21)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct UnloadChunkPacket {
        pub chunk_z: i32,
        pub chunk_x: i32,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "u8")]
    pub enum GameEventType {
        NoRespawnBlockAvailable = 0,
        BeginRaining = 1,
        EndRaining = 2,
        ChangeGameMode = 3,
        WinGame = 4,
        DemoEvent = 5,
        ArrowHitPlayer = 6,
        RainLevelChange = 7,
        ThunderLevelChange = 8,
        PlayPufferfishStingSound = 9,
        PlayElderGuardianMobAppearance = 10,
        EnableRespawnScreen = 11,
        LimitedCrafting = 12,
        StartWaitingForLevelChunks = 13,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x22)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct GameEventPacket {
        pub event: GameEventType,
        pub value: f32,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x23)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct OpenHorseScreenPacket {
        pub window_id: VarInt,
        pub inventory_columns: VarInt,
        pub entity_id: i32,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x24)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct HurtAnimationPacket {
        pub entity_id: VarInt,
        pub yaw: f32,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x25)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct InitializeWorldBorderPacket {
        pub x: f64,
        pub z: f64,
        pub old_diameter: f64,
        pub new_diameter: f64,
        pub speed: i64,
        pub portal_teleport_boundary: VarInt,
        pub warning_blocks: VarInt,
        pub warning_time: VarInt,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x26)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct KeepAlivePacket {
        pub keep_alive_id: i64,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x27)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct ChunkDataAndUpdateLightPacket {
        pub chunk_x: i32,
        pub chunk_z: i32,
        pub data: Vec<u8>,  // TODO: Implement chunk data structure
        pub light: Vec<u8>, // TODO: Implement light data structure
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum PlayerActionStatus {
        StartDigging = 0,
        CancelDigging = 1,
        FinishedDigging = 2,
        DropItemStack = 3,
        DropItem = 4,
        ShootArrowFinishEating = 5,
        SwapItemInHand = 6,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    #[packet(repr = "VarInt")]
    pub enum BlockFace {
        Bottom = 0,
        Top = 1,
        North = 2,
        South = 3,
        West = 4,
        East = 5,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x28)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct PlayerActionPacket {
        pub status: PlayerActionStatus,
        pub position: Position,
        pub face: BlockFace,
        pub sequence: VarInt,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct MapDataIcons {
        pub icon_type: VarInt,
        pub x: i8,
        pub z: i8,
        pub direction: i8,
        pub display_name: Option<String>,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct MapDataColorPatch {
        pub columns: u8,
        pub rows: u8,
        pub x: u8,
        pub z: u8,
        pub data: Vec<u8>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x2C)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct MapDataPacket {
        pub map_id: VarInt,
        pub scale: i8,
        pub locked: bool,
        pub icons: MapDataIcons,
        pub color_patch: MapDataColorPatch,
    }

    #[derive(PacketReadable, PacketWritable, Debug)]
    pub struct MinecartSteps {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub velocity_x: f64,
        pub velocity_y: f64,
        pub velocity_z: f64,
        pub yaw: u8,
        pub pitch: u8,
        pub weight: f32,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x30)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct MoveMinecartAlongTrackPacket {
        pub entity_id: VarInt,
        pub steps: Vec<MinecartSteps>,
    }

    #[derive(Packet, Debug)]
    #[packet_id(0x31)]
    #[packet_state(Play)]
    #[packet_direction(Clientbound)]
    pub struct UpdateEntityRotationPacket {
        pub entity_id: VarInt,
        pub yaw: u8,
        pub pitch: u8,
        pub on_ground: bool,
    }
}
