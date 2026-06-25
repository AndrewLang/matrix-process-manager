export type PerformanceMetric = "cpu" | "gpu" | "memory" | "network" | "disk";

export interface PerformanceNavItem {
    key: PerformanceMetric;
    label: string;
    icon: string;
    accent: string;
}