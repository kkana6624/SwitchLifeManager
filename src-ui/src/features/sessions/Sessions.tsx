import { Container, Title, Table, Text, Badge, Card, Group, Stack, Select } from '@mantine/core';
import { MonitorSharedState, SessionRecord } from '../../types';
import { ORDERED_KEYS } from '../../constants';
import { useState, useMemo } from 'react';

interface SessionsProps {
    state: MonitorSharedState;
}

export function Sessions({ state }: SessionsProps) {
    const [selectedSessionIndex, setSelectedSessionIndex] = useState<string | null>(
        state.recent_sessions.length > 0 ? (state.recent_sessions.length - 1).toString() : null
    );

    const selectedSession = useMemo(() => {
        if (selectedSessionIndex === null) return null;
        const index = parseInt(selectedSessionIndex);
        return state.recent_sessions[index] || null;
    }, [selectedSessionIndex, state.recent_sessions]);

    const sessionOptions = useMemo(() => {
        return state.recent_sessions.map((s, i) => ({
            value: i.toString(),
            label: `${new Date(s.start_time).toLocaleString()} (${s.duration_secs}s)`
        })).reverse();
    }, [state.recent_sessions]);

    const renderSessionStats = (session: SessionRecord) => {
        const stats = session.stats || {};
        
        let totalPresses = 0;
        let totalChatters = 0;
        
        const rows = ORDERED_KEYS.map(key => {
            const keyStats = stats[key] || { presses: 0, chatters: 0 };
            const presses = keyStats.presses;
            const chatters = keyStats.chatters;
            
            totalPresses += presses;
            totalChatters += chatters;
            
            const rate = presses > 0 ? (chatters / presses) * 100 : 0;
            const isHighChatter = rate > 1.0;

            if (presses === 0 && chatters === 0) return null;

            return (
                <Table.Tr key={key}>
                    <Table.Td fw={700}>{key}</Table.Td>
                    <Table.Td>{presses.toLocaleString()}</Table.Td>
                    <Table.Td>
                        {chatters > 0 ? (
                            <Text c={isHighChatter ? 'red' : undefined} fw={isHighChatter ? 700 : undefined}>
                                {chatters.toLocaleString()}
                            </Text>
                        ) : (
                            "-"
                        )}
                    </Table.Td>
                    <Table.Td>
                        {presses > 0 && chatters > 0 ? (
                            <Badge color={isHighChatter ? 'red' : 'gray'} variant="light">
                                {rate.toFixed(2)}%
                            </Badge>
                        ) : (
                            "-"
                        )}
                    </Table.Td>
                </Table.Tr>
            );
        }).filter(row => row !== null);

        return (
            <Stack gap="md">
                <Card shadow="sm" radius="md" withBorder>
                    <Group justify="space-around">
                        <Group>
                            <Text fw={500} c="dimmed">Total Session Presses</Text>
                            <Text size="xl" fw={700}>{totalPresses.toLocaleString()}</Text>
                        </Group>
                        <Group>
                            <Text fw={500} c="dimmed">Total Session Chatters</Text>
                            <Text size="xl" fw={700} c={totalChatters > 0 ? 'red' : undefined}>
                                {totalChatters.toLocaleString()}
                            </Text>
                        </Group>
                    </Group>
                </Card>

                <Card shadow="sm" radius="md" withBorder>
                    <Table striped highlightOnHover>
                        <Table.Thead>
                            <Table.Tr>
                                <Table.Th>Key</Table.Th>
                                <Table.Th>Presses</Table.Th>
                                <Table.Th>Chatters</Table.Th>
                                <Table.Th>Chatter Rate</Table.Th>
                            </Table.Tr>
                        </Table.Thead>
                        <Table.Tbody>
                            {rows.length > 0 ? rows : (
                                <Table.Tr>
                                    <Table.Td colSpan={4} align="center">No activity recorded in this session.</Table.Td>
                                </Table.Tr>
                            )}
                        </Table.Tbody>
                    </Table>
                </Card>
            </Stack>
        );
    };

    return (
        <Container fluid>
            <Group justify="space-between" mb="md">
                <Title order={4}>Past Sessions</Title>
                <Select
                    placeholder="Select a session"
                    data={sessionOptions}
                    value={selectedSessionIndex}
                    onChange={setSelectedSessionIndex}
                    allowDeselect={false}
                    style={{ width: 300 }}
                />
            </Group>

            {selectedSession ? (
                renderSessionStats(selectedSession)
            ) : (
                <Text c="dimmed" ta="center" mt="xl">
                    No session data available. Start playing to record sessions!
                </Text>
            )}
        </Container>
    );
}
