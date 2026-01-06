import { Container, Title, Paper, Group, Stack, Text, Badge, SimpleGrid, Card } from '@mantine/core';
import { MonitorSharedState, SwitchData } from '../../types';
import { ORDERED_KEYS } from '../../constants';

interface LiveSessionProps {
    state: MonitorSharedState;
}

export function LiveSession({ state }: LiveSessionProps) {
    if (!state.is_game_running) {
        return (
            <Container>
                <Title order={4} mb="md">Live Session</Title>
                <Paper p="xl" withBorder style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: 300 }}>
                    <Stack align="center">
                        <Title order={2} c="dimmed">Waiting for Game...</Title>
                        <Text>Start beatmania IIDX INFINITAS to see live stats.</Text>
                    </Stack>
                </Paper>
            </Container>
        );
    }

    // Calculate max presses for scaling
    const maxPresses = ORDERED_KEYS.reduce((max, key) => {
        const stats = state.switches[key]?.stats;
        return Math.max(max, stats?.last_session_presses || 0);
    }, 0);

    return (
        <Container fluid>
            <Group justify="space-between" mb="md">
                <Title order={4}>Live Session Stats</Title>
                <Badge size="lg" color="green" variant="dot">Running</Badge>
            </Group>

            <SimpleGrid cols={{ base: 1, sm: 2, lg: 4 }} spacing="md">
                {ORDERED_KEYS.map(key => {
                    const switchData = state.switches[key] || {
                        stats: { last_session_presses: 0, last_session_chatters: 0 }
                    } as SwitchData;

                    const presses = switchData.stats.last_session_presses;
                    const chatters = switchData.stats.last_session_chatters;

                    // Simple progress calc (scale to max found, avoid div by 0)
                    const progressValue = maxPresses > 0 ? (presses / maxPresses) * 100 : 0;

                    const chatterRate = presses > 0 ? (chatters / presses) * 100 : 0;
                    const isHighChatter = chatterRate > 1.0;

                    return (
                        <Card key={key} shadow="sm" withBorder radius="md">
                            <Stack gap="xs">
                                <Group justify="space-between">
                                    <Title order={3}>{key}</Title>
                                    <Badge color={isHighChatter ? "red" : "gray"} variant="light">
                                        {chatters} chatters
                                    </Badge>
                                </Group>

                                <Group align="flex-end" gap="xs">
                                    <Text size="xl" fw={700} lh={1}>{presses.toLocaleString()}</Text>
                                    <Text size="sm" c="dimmed" mb={2}>presses</Text>
                                </Group>

                                {/* Visual Bar */}
                                <div style={{
                                    height: 8,
                                    backgroundColor: '#eee',
                                    borderRadius: 4,
                                    overflow: 'hidden',
                                    marginTop: 8
                                }}>
                                    <div style={{
                                        height: '100%',
                                        width: `${progressValue}%`,
                                        backgroundColor: 'var(--mantine-color-blue-6)',
                                        transition: 'width 0.2s ease-out'
                                    }} />
                                </div>
                            </Stack>
                        </Card>
                    );
                })}
            </SimpleGrid>
        </Container>
    );
}
