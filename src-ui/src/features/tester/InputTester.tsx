import { Container, Paper, Group, Stack, Text, Title, Grid, Center, Badge } from '@mantine/core';
import { MonitorSharedState } from '../../types';

interface InputTesterProps {
    state: MonitorSharedState;
}

const KeyButton = ({ label, isPressed, size = 60 }: { label: string, isPressed: boolean, size?: number }) => (
    <Paper 
        shadow="md" 
        radius="xl" 
        w={size} 
        h={size}
        bg={isPressed ? 'blue.6' : 'gray.2'}
        style={{ 
            display: 'flex', 
            alignItems: 'center', 
            justifyContent: 'center',
            transition: 'background-color 0.05s',
            border: isPressed ? '2px solid white' : 'none'
        }}
    >
        <Text c={isPressed ? 'white' : 'dimmed'} fw={700}>{label}</Text>
    </Paper>
);

export function InputTester({ state }: InputTesterProps) {
    const isPressed = (key: string) => state.current_pressed_keys.includes(key);

    return (
        <Container>
            <Title order={4} mb="md" ta="center">Input Tester</Title>

            <Center mb="xl">
                <Stack align="center" gap="xl" p="xl" bg="gray.1" style={{ borderRadius: 16 }}>
                    {/* System Buttons (E1-E4) */}
                    <Group gap="lg">
                        <KeyButton label="E1" isPressed={isPressed('E1')} size={40} />
                        <KeyButton label="E2" isPressed={isPressed('E2')} size={40} />
                        <KeyButton label="E3" isPressed={isPressed('E3')} size={40} />
                        <KeyButton label="E4" isPressed={isPressed('E4')} size={40} />
                    </Group>

                    {/* Keys 1-7 */}
                    <Stack gap={-10}> {/* Overlap slightly for visual density or just close gap */}
                        {/* Upper Row: 2, 4, 6 */}
                        <Group gap={40}>
                            <KeyButton label="2" isPressed={isPressed('Key2')} />
                            <KeyButton label="4" isPressed={isPressed('Key4')} />
                            <KeyButton label="6" isPressed={isPressed('Key6')} />
                        </Group>
                        {/* Lower Row: 1, 3, 5, 7 */}
                        <Group gap={40} mt={-30}> {/* Negative margin to nest hex-like */}
                            <KeyButton label="1" isPressed={isPressed('Key1')} />
                            <KeyButton label="3" isPressed={isPressed('Key3')} />
                            <KeyButton label="5" isPressed={isPressed('Key5')} />
                            <KeyButton label="7" isPressed={isPressed('Key7')} />
                        </Group>
                    </Stack>

                    {/* Turntable visualization? Maybe just generic "Other" inputs if mapped? */}
                </Stack>
            </Center>

            <Grid>
                <Grid.Col span={6}>
                    <Paper withBorder p="md">
                        <Text size="sm" c="dimmed">Raw Button State (Bitmask)</Text>
                        <Text ff="monospace" size="xl">{state.raw_button_state} (0x{state.raw_button_state.toString(16).toUpperCase()})</Text>
                    </Paper>
                </Grid.Col>
                <Grid.Col span={6}>
                     <Paper withBorder p="md">
                        <Text size="sm" c="dimmed">Pressed Logical Keys</Text>
                        <Group gap="xs">
                            {state.current_pressed_keys.length > 0 ? (
                                state.current_pressed_keys.map(k => <Badge key={k}>{k}</Badge>)
                            ) : (
                                <Text c="dimmed" fs="italic">None</Text>
                            )}
                        </Group>
                    </Paper>
                </Grid.Col>
            </Grid>
        </Container>
    );
}
