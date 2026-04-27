import { SWITCH_MODELS } from '../../constants';

export const getSwitchModel = (id: string): { id: string, name: string, rated_lifespan_presses: number } => {
    return SWITCH_MODELS.find(m => m.id === id) || SWITCH_MODELS.find(m => m.id === "generic_unknown")!;
};

export const getLifeExpectancyPercentage = (presses: number, rated: number): number => {
    const remaining = rated - presses;
    if (remaining <= 0) return 0;
    return (remaining / rated) * 100;
};

export const getProgressColor = (percentage: number): string => {
    if (percentage > 50) return 'green';
    if (percentage > 25) return 'yellow';
    return 'red';
};

export const getLocalISOTime = (dateString?: string | null) => {
    const date = dateString ? new Date(dateString) : new Date();
    const offsetMs = date.getTimezoneOffset() * 60 * 1000;
    return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16);
};
