import { Container, Title, Table } from '@mantine/core';
import { MonitorSharedState } from '../../types';

interface HistoryProps {
    state: MonitorSharedState;
}

export function History({ state }: HistoryProps) {
    // Sort by date desc
    const sortedHistory = [...state.switch_history].sort((a, b) => 
        new Date(b.date).getTime() - new Date(a.date).getTime()
    );

    return (
        <Container fluid>
            <Title order={4} mb="md">Replacement History</Title>
            <Table striped highlightOnHover>
                <Table.Thead>
                    <Table.Tr>
                        <Table.Th>Date</Table.Th>
                        <Table.Th>Key</Table.Th>
                        <Table.Th>Event</Table.Th>
                        <Table.Th>Model Change</Table.Th>
                        <Table.Th>Previous Stats (Press/Chatter)</Table.Th>
                    </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                    {sortedHistory.map((entry, index) => (
                        <Table.Tr key={index}>
                            <Table.Td>{new Date(entry.date).toLocaleString()}</Table.Td>
                            <Table.Td>{entry.key}</Table.Td>
                            <Table.Td>{entry.event_type}</Table.Td>
                            <Table.Td>
                                {entry.old_model_id !== entry.new_model_id 
                                    ? `${entry.old_model_id} -> ${entry.new_model_id}` 
                                    : entry.new_model_id}
                            </Table.Td>
                            <Table.Td>
                                {entry.previous_stats.total_presses.toLocaleString()} / {entry.previous_stats.total_chatters.toLocaleString()}
                            </Table.Td>
                        </Table.Tr>
                    ))}
                    {sortedHistory.length === 0 && (
                        <Table.Tr>
                            <Table.Td colSpan={5} align="center">No history recorded.</Table.Td>
                        </Table.Tr>
                    )}
                </Table.Tbody>
            </Table>
        </Container>
    );
}
