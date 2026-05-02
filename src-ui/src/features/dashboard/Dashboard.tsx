import { Container, Grid, Group, Select, Button, Title, Paper } from '@mantine/core';
import { MonitorSharedState, SwitchData } from '../../types';
import { ORDERED_KEYS } from '../../constants';
import { useDashboard } from './useDashboard';
import { SwitchCard } from './components/SwitchCard';
import { SessionInfoPanel } from './components/SessionInfoPanel';
import { BulkActionPanel } from './components/BulkActionPanel';
import { DateEditModal } from './components/DateEditModal';

interface DashboardProps {
    state: MonitorSharedState;
}

export function Dashboard({ state }: DashboardProps) {
    const {
        selectedKeys,
        bulkModelId,
        setBulkModelId,
        dateModalOpen,
        setDateModalOpen,
        editingKey,
        editingDate,
        setEditingDate,
        handleResetStats,
        handleModelChange,
        toggleSelection,
        toggleSelectAll,
        handleBulkReset,
        handleBulkApplyModel,
        handleOpenDateEdit,
        handleSaveDate,
        handleControllerChange,
        controllerOptions
    } = useDashboard(state);

    return (
        <Container fluid>
            <Paper p="md" mb="md" withBorder>
                <Group justify="space-between">
                    <Title order={4}>Switch Life Expectancy</Title>
                    <Group>
                        <Select
                            label="Active Controller"
                            placeholder="Select Controller"
                            data={controllerOptions}
                            value={state.active_controller_id}
                            onChange={handleControllerChange}
                            style={{ minWidth: 250 }}
                        />
                        <Button variant="default" size="xs" onClick={toggleSelectAll} mt={24}>
                            {selectedKeys.length === ORDERED_KEYS.length ? "Deselect All" : "Select All"}
                        </Button>
                    </Group>
                </Group>
            </Paper>

            <BulkActionPanel
                selectedKeysCount={selectedKeys.length}
                bulkModelId={bulkModelId}
                onBulkModelIdChange={setBulkModelId}
                onApplyModel={handleBulkApplyModel}
                onResetStats={handleBulkReset}
            />

            <SessionInfoPanel state={state} />

            <Grid>
                {ORDERED_KEYS.map(key => {
                    const switchData = state.switches[key] || {
                        switch_model_id: "generic_unknown",
                        stats: {
                            total_presses: 0,
                            total_releases: 0,
                            total_chatters: 0,
                            total_chatter_releases: 0,
                            last_session_presses: 0,
                            last_session_chatters: 0,
                            last_session_chatter_releases: 0
                        }
                    } as SwitchData;

                    return (
                        <SwitchCard
                            key={key}
                            switchKey={key}
                            switchData={switchData}
                            isSelected={selectedKeys.includes(key)}
                            onToggleSelect={() => toggleSelection(key)}
                            onReset={() => handleResetStats(key)}
                            onModelChange={(modelId) => handleModelChange(key, modelId)}
                            onEditDate={() => handleOpenDateEdit(key, switchData.last_replaced_at)}
                        />
                    );
                })}
            </Grid>

            <DateEditModal
                opened={dateModalOpen}
                onClose={() => setDateModalOpen(false)}
                editingKey={editingKey}
                editingDate={editingDate}
                onDateChange={setEditingDate}
                onSave={handleSaveDate}
            />
        </Container>
    );
}