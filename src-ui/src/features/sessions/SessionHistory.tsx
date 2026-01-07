import { Container, Title, Grid, Paper, Stack, Text, Group, Badge, ScrollArea, Card, Table, Loader, Center } from '@mantine/core';
import { MonitorSharedState, SessionRecord, SessionKeyStats } from '../../types';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SessionHistoryProps {
    state: MonitorSharedState;
}

export function SessionHistory({ state }: SessionHistoryProps) {
    const [historySessions, setHistorySessions] = useState<SessionRecord[]>([]);
    const [selectedSession, setSelectedSession] = useState<SessionRecord | null>(null);
    const [sessionDetails, setSessionDetails] = useState<SessionKeyStats[] | null>(null);
    const [isLoadingHistory, setIsLoadingHistory] = useState(false);
    const [isLoadingDetails, setIsLoadingDetails] = useState(false);

    // Generate a dependency key that only changes when the session list content changes
    const recentSessionKey = state.recent_sessions
        ? `${state.recent_sessions.length}_${state.recent_sessions[state.recent_sessions.length - 1]?.end_time}`
        : "";

    // Fetch history on mount and when a new session might have been added
    useEffect(() => {
        const fetchHistory = async () => {
            setIsLoadingHistory(true);
            try {
                const sessions = await invoke<SessionRecord[]>('get_history_sessions', { limit: 50, offset: 0 });
                setHistorySessions(sessions);
                // Auto-select the latest session if none selected
                if (!selectedSession && sessions.length > 0) {
                    handleSelectSession(sessions[0]);
                }
            } catch (error) {
                console.error("Failed to fetch session history:", error);
            } finally {
                setIsLoadingHistory(false);
            }
        };

        fetchHistory();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [recentSessionKey]);

    const handleSelectSession = async (session: SessionRecord) => {
        setSelectedSession(session);
        setSessionDetails(null); // Clear previous details while loading

        if (session.id === undefined) {
            // Fallback for sessions without ID (should not happen with DB persistance)
            return;
        }

        setIsLoadingDetails(true);
        try {
            const details = await invoke<SessionKeyStats[]>('get_session_details', { sessionId: session.id });
            setSessionDetails(details);
        } catch (error) {
            console.error("Failed to fetch session details:", error);
        } finally {
            setIsLoadingDetails(false);
        }
    };

    return (
        <Container fluid>
            <Title order={4} mb="md">Past Sessions (History)</Title>
            <Grid>
                {/* Left Pane: Session List */}
                <Grid.Col span={{ base: 12, md: 4 }}>
                    <Paper withBorder h={600} display="flex" style={{ flexDirection: 'column' }}>
                        <Text p="xs" fw={700} bg="gray.1" style={{ borderBottom: '1px solid #eee' }}>
                            Session Log
                        </Text>
                        <ScrollArea style={{ flex: 1 }}>
                            {isLoadingHistory && (
                                <Center p="xl">
                                    <Loader size="sm" />
                                </Center>
                            )}
                            {!isLoadingHistory && historySessions.length === 0 && (
                                <Text p="md" c="dimmed" ta="center">No sessions recorded.</Text>
                            )}
                            {historySessions.map((session, index) => {
                                const isSelected = selectedSession?.id === session.id;
                                return (
                                    <div
                                        key={session.id || index}
                                        onClick={() => handleSelectSession(session)}
                                        style={{
                                            padding: '12px',
                                            cursor: 'pointer',
                                            backgroundColor: isSelected ? 'var(--mantine-color-blue-0)' : 'transparent',
                                            borderBottom: '1px solid #f0f0f0',
                                            transition: 'background-color 0.2s'
                                        }}
                                    >
                                        <Group justify="space-between" mb={4}>
                                            <Text size="sm" fw={500}>
                                                {new Date(session.start_time).toLocaleString()}
                                            </Text>
                                        </Group>
                                        <Group gap="xs">
                                            <Badge size="sm" variant="outline">
                                                {session.duration_secs}s
                                            </Badge>
                                            {index === 0 && <Badge size="sm" color="green">Latest</Badge>}
                                        </Group>
                                    </div>
                                );
                            })}
                        </ScrollArea>
                    </Paper>
                </Grid.Col>

                {/* Right Pane: Details */}
                <Grid.Col span={{ base: 12, md: 8 }}>
                    {selectedSession ? (
                        <Card shadow="sm" radius="md" withBorder h="100%">
                            <Stack>
                                <Group justify="space-between">
                                    <Title order={5}>Session Details</Title>
                                    <Stack gap={0} align="flex-end">
                                        <Text size="sm" c="dimmed">
                                            Start: {new Date(selectedSession.start_time).toLocaleString()}
                                        </Text>
                                        <Text size="sm" c="dimmed">
                                            End: {new Date(selectedSession.end_time).toLocaleTimeString()}
                                        </Text>
                                    </Stack>
                                </Group>

                                <Paper p="md" bg="gray.0">
                                    <Group>
                                        <Text>Duration:</Text>
                                        <Badge size="lg" variant="filled">{selectedSession.duration_secs} Seconds</Badge>
                                    </Group>
                                </Paper>

                                <Title order={6} mt="sm">Key Statistics</Title>

                                {isLoadingDetails ? (
                                    <Center p="xl"><Loader /></Center>
                                ) : sessionDetails ? (
                                    <ScrollArea h={400}>
                                        <Table striped highlightOnHover>
                                            <Table.Thead>
                                                <Table.Tr>
                                                    <Table.Th>Key</Table.Th>
                                                    <Table.Th style={{ textAlign: 'right' }}>Presses</Table.Th>
                                                    <Table.Th style={{ textAlign: 'right' }}>Chatters</Table.Th>
                                                    <Table.Th style={{ textAlign: 'right' }}>Chatter Rate</Table.Th>
                                                </Table.Tr>
                                            </Table.Thead>
                                            <Table.Tbody>
                                                {sessionDetails.sort((a, b) => a.key_name.localeCompare(b.key_name, undefined, { numeric: true })).map((stat) => {
                                                    const rate = stat.presses > 0
                                                        ? ((stat.chatters / stat.presses) * 100).toFixed(2)
                                                        : "0.00";

                                                    // Highlight problematic keys
                                                    const isHighChatter = parseFloat(rate) > 1.0 && stat.presses > 10;

                                                    return (
                                                        <Table.Tr key={stat.key_name}>
                                                            <Table.Td fw={500}>{stat.key_name}</Table.Td>
                                                            <Table.Td style={{ textAlign: 'right' }}>{stat.presses.toLocaleString()}</Table.Td>
                                                            <Table.Td style={{ textAlign: 'right' }} c={isHighChatter ? 'red' : undefined} fw={isHighChatter ? 700 : 400}>
                                                                {stat.chatters.toLocaleString()}
                                                            </Table.Td>
                                                            <Table.Td style={{ textAlign: 'right' }}>{rate}%</Table.Td>
                                                        </Table.Tr>
                                                    );
                                                })}
                                            </Table.Tbody>
                                        </Table>
                                    </ScrollArea>
                                ) : (
                                    <Text c="dimmed">No details available for this session.</Text>
                                )}
                            </Stack>
                        </Card>
                    ) : (
                        <Paper p="xl" withBorder h="100%" style={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                            <Text c="dimmed">Select a session to view details</Text>
                        </Paper>
                    )}
                </Grid.Col>
            </Grid>
        </Container>
    );
}
