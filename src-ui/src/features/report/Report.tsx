import { Container, Title, Table, Text, Badge, Card, Group } from '@mantine/core';
import { MonitorSharedState, SwitchData } from '../../types';
import { ORDERED_KEYS } from '../../constants';

interface ReportProps {
    state: MonitorSharedState;
}

export function Report({ state }: ReportProps) {
    // Calculate totals
    let totalSessionPresses = 0;
    let totalSessionChatters = 0;

    const rows = ORDERED_KEYS.map(key => {
        const switchData = state.switches[key] || {
            stats: { last_session_presses: 0, last_session_chatters: 0 }
        } as SwitchData;
        
        const presses = switchData.stats.last_session_presses;
        const chatters = switchData.stats.last_session_chatters;
        
        totalSessionPresses += presses;
        totalSessionChatters += chatters;

        const rate = presses > 0 ? (chatters / presses) * 100 : 0;
        const isHighChatter = rate > 1.0; // Highlight if > 1%

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
    });

    return (
        <Container>
            <Title order={4} mb="md">Session Report</Title>
            
            <Card shadow="sm" radius="md" withBorder mb="lg">
                <Group justify="space-around">
                    <Group>
                        <Text fw={500} c="dimmed">Total Presses</Text>
                        <Text size="xl" fw={700}>{totalSessionPresses.toLocaleString()}</Text>
                    </Group>
                    <Group>
                        <Text fw={500} c="dimmed">Total Chatters</Text>
                        <Text size="xl" fw={700} c={totalSessionChatters > 0 ? 'red' : undefined}>
                            {totalSessionChatters.toLocaleString()}
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
                    <Table.Tbody>{rows}</Table.Tbody>
                </Table>
            </Card>
        </Container>
    );
}
