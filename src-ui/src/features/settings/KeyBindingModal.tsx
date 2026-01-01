import { Modal, Text, Stack, Button, Code, Loader } from '@mantine/core';
import { useEffect, useState } from 'react';

interface KeyBindingModalProps {
    opened: boolean;
    onClose: () => void;
    targetKey: string | null;
    currentRawState: number;
    onBind: (key: string, button: number) => void;
}

export function KeyBindingModal({ opened, onClose, targetKey, currentRawState, onBind }: KeyBindingModalProps) {
    // We want to detect the *first* press after modal opens.
    // However, if the user is holding a button *before* opening, we might want to wait for release first (debounce).
    // For simplicity, we just look for currentRawState > 0.
    
    // Ideally:
    // 1. Wait for rawState == 0 (Release all)
    // 2. Wait for rawState > 0 (Press)
    // 3. Take that value.
    
    const [step, setStep] = useState<'wait-release' | 'wait-press'>('wait-release');

    useEffect(() => {
        if (!opened) {
            setStep('wait-release');
            return;
        }

        if (step === 'wait-release') {
            if (currentRawState === 0) {
                setStep('wait-press');
            }
        } else if (step === 'wait-press') {
            if (currentRawState > 0) {
                // Detected press!
                // We could delay slightly to ensure stability or allow combos, but usually it's single button.
                if (targetKey) {
                    onBind(targetKey, currentRawState);
                    onClose();
                }
            }
        }
    }, [opened, currentRawState, step, targetKey, onBind, onClose]);

    return (
        <Modal opened={opened} onClose={onClose} title="Key Binding Learning" centered>
            <Stack align="center" py="xl">
                {step === 'wait-release' ? (
                    <>
                        <Text size="lg" fw={500}>Please release all buttons...</Text>
                        <Loader color="yellow" type="dots" />
                    </>
                ) : (
                    <>
                        <Text size="lg" fw={500}>Press the button for <Code fz="lg">{targetKey}</Code></Text>
                        <Loader color="blue" variant="bars" />
                        <Text size="sm" c="dimmed">Waiting for input...</Text>
                    </>
                )}
                
                <Button variant="subtle" color="gray" onClick={onClose} mt="md">
                    Cancel
                </Button>
            </Stack>
        </Modal>
    );
}
