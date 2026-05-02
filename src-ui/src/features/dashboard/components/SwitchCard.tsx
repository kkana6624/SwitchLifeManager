import { Grid, Card, Group, Text, Checkbox, Badge, Progress, Stack, Button, Select } from '@mantine/core';
import { SwitchData } from '../../../types';
import { SWITCH_MODELS } from '../../../constants';
import { getSwitchModel, getLifeExpectancyPercentage, getProgressColor } from '../utils';

interface SwitchCardProps {
    switchKey: string;
    switchData: SwitchData;
    isSelected: boolean;
    onToggleSelect: () => void;
    onReset: () => void;
    onModelChange: (modelId: string) => void;
    onEditDate: () => void;
}

export function SwitchCard({
    switchKey,
    switchData,
    isSelected,
    onToggleSelect,
    onReset,
    onModelChange,
    onEditDate
}: SwitchCardProps) {
    const model = getSwitchModel(switchData.switch_model_id);
    const percentage = getLifeExpectancyPercentage(switchData.stats.total_presses, model.rated_lifespan_presses);
    const color = getProgressColor(percentage);

    const totalEvents = switchData.stats.total_presses + switchData.stats.total_chatters;
    const chatterRate = totalEvents > 0 ? (switchData.stats.total_chatters / totalEvents) * 100 : 0;
    const isHighChatter = chatterRate > 0.5;

    return (
        <Grid.Col span={{ base: 12, md: 6, lg: 4 }}>
            <Card
                shadow="sm"
                padding="lg"
                radius="md"
                withBorder
                style={{ borderColor: isSelected ? 'var(--mantine-color-blue-5)' : undefined, borderWidth: isSelected ? 2 : 1 }}
            >
                <Group justify="space-between" mb="xs">
                    <Group>
                        <Checkbox
                            checked={isSelected}
                            onChange={onToggleSelect}
                        />
                        <Text fw={700}>{switchKey}</Text>
                    </Group>
                    <Group gap="xs">
                        {isHighChatter && (
                            <Badge color="red" variant="filled">Warning: Chatter</Badge>
                        )}
                        <Badge color={color} variant="light">
                            {percentage.toFixed(1)}% Life
                        </Badge>
                    </Group>
                </Group>

                <Text size="sm" c="dimmed" mb="xs">
                    Model: {model.name}
                </Text>

                <Progress
                    value={percentage}
                    color={color}
                    size="xl"
                    radius="xl"
                    mb="md"
                />

                <Group grow mb="md">
                    <Stack gap={0}>
                        <Text size="xs" c="dimmed">Presses</Text>
                        <Text fw={500}>{switchData.stats.total_presses.toLocaleString()}</Text>
                    </Stack>
                    <Stack gap={0}>
                        <Text size="xs" c="dimmed">Chatters</Text>
                        <Text fw={500} c={isHighChatter ? 'red' : undefined}>
                            {switchData.stats.total_chatters.toLocaleString()} ({chatterRate.toFixed(2)}%)
                        </Text>
                    </Stack>
                    <Stack gap={0}>
                        <Text size="xs" c="dimmed">Session</Text>
                        <Text fw={500}>
                            {switchData.stats.last_session_presses.toLocaleString()}
                        </Text>
                    </Stack>
                </Group>

                <Stack gap={0} mb="md">
                    <Text size="xs" c="dimmed">Last Replaced</Text>
                    <Group justify="space-between">
                        <Text size="sm">
                            {switchData.last_replaced_at ? new Date(switchData.last_replaced_at).toLocaleDateString() : 'Never'}
                        </Text>
                        <Button variant="subtle" size="compact-xs" onClick={onEditDate}>
                            Edit
                        </Button>
                    </Group>
                </Stack>

                <Group>
                    <Select
                        size="xs"
                        data={SWITCH_MODELS.map(m => ({ value: m.id, label: m.name }))}
                        value={switchData.switch_model_id}
                        onChange={(val: string | null) => val && onModelChange(val)}
                        style={{ flexGrow: 1 }}
                    />
                    <Button size="xs" color="red" variant="light" onClick={onReset}>
                        Reset
                    </Button>
                </Group>
            </Card>
        </Grid.Col>
    );
}
