import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { MonitorSharedState } from '../../types';
import { ORDERED_KEYS } from '../../constants';
import { getLocalISOTime } from './utils';

export function useDashboard(state: MonitorSharedState) {
    const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
    const [bulkModelId, setBulkModelId] = useState<string | null>(null);

    // Date Edit State
    const [dateModalOpen, setDateModalOpen] = useState(false);
    const [editingKey, setEditingKey] = useState<string | null>(null);
    const [editingDate, setEditingDate] = useState<string>('');

    const handleResetStats = (key: string) => {
        if (window.confirm(`Reset stats for ${key}?`)) {
            void invoke('reset_stats', { key });
        }
    };

    const handleModelChange = (key: string, modelId: string) => {
        if (window.confirm(`Change model for ${key} to ${modelId}? Stats will be reset.`)) {
            void invoke('replace_switch', { key, newModelId: modelId });
        }
    };

    const toggleSelection = (key: string) => {
        setSelectedKeys(prev =>
            prev.includes(key) ? prev.filter(k => k !== key) : [...prev, key]
        );
    };

    const toggleSelectAll = () => {
        if (selectedKeys.length === ORDERED_KEYS.length) {
            setSelectedKeys([]);
        } else {
            setSelectedKeys([...ORDERED_KEYS]);
        }
    };

    const handleBulkReset = () => {
        if (selectedKeys.length === 0) return;
        if (window.confirm(`Reset stats for ${selectedKeys.length} selected keys?`)) {
            selectedKeys.forEach(key => { void invoke('reset_stats', { key }); });
            setSelectedKeys([]);
        }
    };

    const handleBulkApplyModel = () => {
        if (selectedKeys.length === 0 || !bulkModelId) return;
        if (window.confirm(`Change model to ${bulkModelId} for ${selectedKeys.length} keys? Stats will be reset.`)) {
            selectedKeys.forEach(key => { void invoke('replace_switch', { key, newModelId: bulkModelId }); });
            setSelectedKeys([]);
        }
    };

    const handleOpenDateEdit = (key: string, currentDate: string | null) => {
        setEditingKey(key);
        setEditingDate(getLocalISOTime(currentDate));
        setDateModalOpen(true);
    };

    const handleSaveDate = () => {
        if (editingKey && editingDate) {
            const dateObj = new Date(editingDate);
            void invoke('set_last_replaced_date', { key: editingKey, date: dateObj.toISOString() });
            setDateModalOpen(false);
        }
    };

    const handleControllerChange = (id: string | null) => {
        if (id) {
            void invoke('set_active_controller', { id });
        }
    };

    const connected = state.connected_controllers ?? [];
    const controllerOptions = connected.map(c => ({
        value: c.id,
        label: `${c.name} (${c.id.substring(0, 8)})`
    }));

    if (state.active_controller_id && !controllerOptions.find(o => o.value === state.active_controller_id)) {
        controllerOptions.push({
            value: state.active_controller_id,
            label: `Disconnected Controller (${state.active_controller_id.substring(0, 8)})`
        });
    }

    return {
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
    };
}
