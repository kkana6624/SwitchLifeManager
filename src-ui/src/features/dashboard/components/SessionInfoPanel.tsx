import { Grid, Paper, Group, Title, Badge, Stack, Text } from '@mantine/core';
import { MonitorSharedState } from '../../../types';
import { ORDERED_KEYS } from '../../../constants';

interface SessionInfoPanelProps {
    state: MonitorSharedState;
}

export function SessionInfoPanel({ state }: SessionInfoPanelProps) {
    return (
        <Grid mb="lg">
            <Grid.Col span={{ base: 12, md: 6 }}>
                <Paper shadow="xs" p="md" withBorder h="100%">
                    <Group justify="space-between" mb="xs">
                        <Title order={5}>Session Stats ({state.is_game_running ? "Live" : "Previous"})</Title>
                        {state.is_game_running && <Badge color="green" variant="dot">Running</Badge>}
                    </Group>
                    <Group grow>
                        <Stack gap={0}>
                            <Text size="xs" c="dimmed">Session Presses</Text>
                            <Text fw={700} size="lg">
                                {ORDERED_KEYS.reduce((acc, key) => acc + (state.switches[key]?.stats.last_session_presses || 0), 0).toLocaleString()}
                            </Text>
                        </Stack>
                        <Stack gap={0}>
                            <Text size="xs" c="dimmed">Session Chatters</Text>
                            <Text fw={700} size="lg" c="red">
                                {ORDERED_KEYS.reduce((acc, key) => acc + (state.switches[key]?.stats.last_session_chatters || 0), 0).toLocaleString()}
                            </Text>
                        </Stack>
                    </Group>
                </Paper>
            </Grid.Col>

            <Grid.Col span={{ base: 12, md: 6 }}>
                <Paper shadow="xs" p="md" withBorder h="100%">
                    <Title order={5} mb="xs">Recent Sessions (Last 3)</Title>
                    <Stack gap="xs">
                        {[0, 1, 2].map((i) => {
                            const recent = state.recent_sessions ?? [];
                            const session = recent.length > 0 ? [...recent].reverse()[i] : undefined;

                            return (
                                <Group key={i} justify="space-between" h={28}>
                                    <Text size="sm">
                                        {session ? new Date(session.start_time).toLocaleString() : "-"}
                                    </Text>
                                    <Badge variant={session ? "outline" : "transparent"} c={session ? undefined : "dimmed"}>
                                        {session ? `${session.duration_secs}s` : "-"}
                                    </Badge>
                                </Group>
                            );
                        })}
                    </Stack>
                </Paper>
            </Grid.Col>
        </Grid>
    );
}
