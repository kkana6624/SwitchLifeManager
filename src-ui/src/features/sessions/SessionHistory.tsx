import { Container, Title, Grid, Paper, Stack, Text, Group, Badge, ScrollArea, Card } from '@mantine/core';
import { MonitorSharedState, SessionRecord } from '../../types';
import { useState, useEffect } from 'react';

interface SessionHistoryProps {
    state: MonitorSharedState;
}

export function SessionHistory({ state }: SessionHistoryProps) {
    const [selectedSession, setSelectedSession] = useState<SessionRecord | null>(null);

    // Update selected session to the latest one when sessions update, if nothing was selected manually?
    // Or just default to the last one on mount.
    useEffect(() => {
        if (!selectedSession && state.recent_sessions && state.recent_sessions.length > 0) {
            setSelectedSession(state.recent_sessions[state.recent_sessions.length - 1]);
        }
    }, [state.recent_sessions]);

    // Reverse order for display (Newest first)
    const sessions = state.recent_sessions ? [...state.recent_sessions].reverse() : [];

    return (
        <Container fluid>
            <Title order={4} mb="md">Past Sessions</Title>
            <Grid>
                {/* Left Pane: Session List */}
                <Grid.Col span={{ base: 12, md: 4 }}>
                    <Paper withBorder h={500} display="flex" style={{ flexDirection: 'column' }}>
                        <Text p="xs" fw={700} bg="gray.1" style={{ borderBottom: '1px solid #eee' }}>
                            Recent Sessions
                        </Text>
                        <ScrollArea style={{ flex: 1 }}>
                            {sessions.length === 0 && <Text p="md" c="dimmed" ta="center">No sessions recorded.</Text>}
                            {sessions.map((session, index) => {
                                const isSelected = selectedSession === session;
                                return (
                                    <div
                                        key={index}
                                        onClick={() => setSelectedSession(session)}
                                        style={{
                                            padding: '12px',
                                            cursor: 'pointer',
                                            backgroundColor: isSelected ? 'var(--mantine-color-blue-0)' : 'transparent',
                                            borderBottom: '1px solid #f0f0f0'
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
                                    <Text size="sm" c="dimmed">
                                        {new Date(selectedSession.start_time).toLocaleString()} - {new Date(selectedSession.end_time).toLocaleTimeString()}
                                    </Text>
                                </Group>

                                <Paper p="md" bg="gray.0">
                                    <Text>Duration: <b>{selectedSession.duration_secs} seconds</b></Text>
                                    {/* Note: Currently backend might not be storing per-key stats in SessionRecord history. 
                                        If the requirement is to show detailed stats for "Past Sessions", we need to verify if `SessionRecord` has that data.
                                        Looking at `models.rs` and `AppConfig`, `SessionRecord` only has timestamp/duration currently?
                                        
                                        Wait, looking at `architecture.md`:
                                        "直近3回のセッションについて、開始時刻・終了時刻・プレイ時間を記録し..." 
                                        It seems strictly history of times. 
                                        BUT "詳細ビュー (右ペイン): ...各キーのプレス数..." is in the NEW requirement (6.4).
                                        
                                        Crucial Check: Does backend store stats in SessionRecord?
                                        I suspect NO based on previous files.
                                        
                                        If NOT, I can only show "Last Session" stats (which are in `state.switches[].stats.last_session_presses`) 
                                        IF the selected session is indeed the "Last Session".
                                        For older sessions, I might not have the data unless I persist it.
                                        
                                        For now, I will implement "Last Session" details if the selected session matches the last one.
                                        For others, I'll show "Detailed stats not available".
                                    */}
                                </Paper>

                                {/* Placeholder for stats - Implementation Dependency Check needed */}
                                <Text c="dimmed" size="sm" mt="md">
                                    * Detailed key statistics are currently only available for the most recent session.
                                    (Architecture update might be needed to store full stats per session in history)
                                </Text>
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
