export interface SwitchModelInfo {
    id: string;
    name: string;
    manufacturer: string;
    rated_lifespan_presses: number;
}

export const SWITCH_MODELS: SwitchModelInfo[] = [
    {
        id: "omron_d2mv_01_1c3",
        name: "D2MV-01-1C3 (50g)",
        manufacturer: "Omron",
        rated_lifespan_presses: 10_000_000,
    },
    {
        id: "omron_d2mv_01_1c2",
        name: "D2MV-01-1C2 (25g)",
        manufacturer: "Omron",
        rated_lifespan_presses: 10_000_000,
    },
    {
        id: "omron_v_10_1a4",
        name: "V-10-1A4 (100g)",
        manufacturer: "Omron",
        rated_lifespan_presses: 50_000_000,
    },
    {
        id: "generic_unknown",
        name: "Generic / Unknown",
        manufacturer: "Generic",
        rated_lifespan_presses: 1_000_000,
    },
];

export const ORDERED_KEYS = [
    "Key1", "Key2", "Key3", "Key4", "Key5", "Key6", "Key7",
    "E1", "E2", "E3", "E4"
];
