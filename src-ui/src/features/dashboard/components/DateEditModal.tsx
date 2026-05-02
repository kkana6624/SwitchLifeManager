import { Modal, Stack, TextInput, Group, Button } from '@mantine/core';
import { ChangeEvent } from 'react';

interface DateEditModalProps {
    opened: boolean;
    onClose: () => void;
    editingKey: string | null;
    editingDate: string;
    onDateChange: (date: string) => void;
    onSave: () => void;
}

export function DateEditModal({
    opened,
    onClose,
    editingKey,
    editingDate,
    onDateChange,
    onSave
}: DateEditModalProps) {
    return (
        <Modal opened={opened} onClose={onClose} title={`Set Last Replaced Date: ${editingKey}`}>
            <Stack>
                <TextInput
                    type="datetime-local"
                    label="Replacement Date"
                    value={editingDate}
                    onChange={(e: ChangeEvent<HTMLInputElement>) => onDateChange(e.currentTarget.value)}
                />
                <Group justify="flex-end">
                    <Button variant="default" onClick={onClose}>Cancel</Button>
                    <Button onClick={onSave}>Save</Button>
                </Group>
            </Stack>
        </Modal>
    );
}
