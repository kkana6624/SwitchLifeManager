import { Paper, Title, Group, Select, Button } from '@mantine/core';
import { SWITCH_MODELS } from '../../../constants';

interface BulkActionPanelProps {
    selectedKeysCount: number;
    bulkModelId: string | null;
    onBulkModelIdChange: (val: string | null) => void;
    onApplyModel: () => void;
    onResetStats: () => void;
}

export function BulkActionPanel({
    selectedKeysCount,
    bulkModelId,
    onBulkModelIdChange,
    onApplyModel,
    onResetStats
}: BulkActionPanelProps) {
    if (selectedKeysCount === 0) return null;

    return (
        <Paper p="md" mb="lg" bg="blue.0" withBorder>
            <Title order={5} mb="xs">Bulk Actions ({selectedKeysCount} selected)</Title>
            <Group align="end">
                <Select
                    label="Change Model"
                    placeholder="Select Switch Model"
                    data={SWITCH_MODELS.map(m => ({ value: m.id, label: m.name }))}
                    value={bulkModelId}
                    onChange={onBulkModelIdChange}
                    style={{ flexGrow: 1, maxWidth: 300 }}
                />
                <Button onClick={onApplyModel} disabled={!bulkModelId}>
                    Apply Model
                </Button>
                <Button color="red" variant="light" onClick={onResetStats}>
                    Reset Stats
                </Button>
            </Group>
        </Paper>
    );
}
