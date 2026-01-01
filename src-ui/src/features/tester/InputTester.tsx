import { Container, Paper, Group, Text, Title, Grid, Center, Badge, Box } from '@mantine/core';
import { MonitorSharedState } from '../../types';

interface InputTesterProps {
    state: MonitorSharedState;
}

interface KeyButtonProps {
    label: string;
    isPressed: boolean;
    left?: number;
    top?: number;
    width?: number;
    height?: number;
    isBlackKey?: boolean;
}

const KeyButton = ({ label, isPressed, left, top, width = 50, height = 80, isBlackKey = false }: KeyButtonProps) => {
    // Colors
    // Pressed: Red for all (standard IIDX light color) or Blue. Let's use Red/Pink.
    // Idle: White keys -> Gray/White, Black keys -> Black/DarkGray.
    
    let bg = isBlackKey ? 'dark.6' : 'gray.0';
    let color = isBlackKey ? 'white' : 'dark';
    let borderColor = isBlackKey ? 'gray.7' : 'gray.4';

    if (isPressed) {
        bg = 'red.6';
        color = 'white';
        borderColor = 'red.8';
    }

    return (
        <Paper 
            shadow="md" 
            radius="sm" 
            w={width} 
            h={height}
            bg={bg}
            style={{ 
                position: left !== undefined ? 'absolute' : 'relative',
                left,
                top,
                display: 'flex', 
                alignItems: 'flex-end', 
                justifyContent: 'center',
                transition: 'background-color 0.05s, transform 0.05s',
                border: `2px solid var(--mantine-color-${borderColor})`,
                zIndex: isBlackKey ? 2 : 1, // Black keys usually visually on top? Or distinct.
                paddingBottom: 8,
                transform: isPressed ? 'scale(0.95)' : 'none',
            }}
        >
            <Text c={color} fw={700} size="sm">{label}</Text>
        </Paper>
    );
};

export function InputTester({ state }: InputTesterProps) {
    const isPressed = (key: string) => state.current_pressed_keys.includes(key);

    return (
        <Container>
            <Title order={4} mb="md" ta="center">Input Tester</Title>

            <Center mb="xl">
                <Box p="xl" bg="gray.2" style={{ borderRadius: 16 }}>
                    {/* IIDX Keyboard Layout */}
                    {/* Adjusted dimensions to fit 7 keys (width 50) with 35px center-to-center offset.
                        Visual center of assembly: 140px. Total width approx 280px.
                     */}
                    <Box pos="relative" w={280} h={220} bg="gray.3" style={{ borderRadius: 8, border: '1px solid #ccc' }}>
                        {/* System Buttons (E1-E4) - Aligned with White Key columns */}
                        <KeyButton label="E1" isPressed={isPressed('E1')} left={15} top={10} width={40} height={35} />
                        <KeyButton label="E2" isPressed={isPressed('E2')} left={85} top={10} width={40} height={35} />
                        <KeyButton label="E3" isPressed={isPressed('E3')} left={155} top={10} width={40} height={35} />
                        <KeyButton label="E4" isPressed={isPressed('E4')} left={225} top={10} width={40} height={35} />

                        {/* Black Keys (Top row of keyboard) */}
                        <KeyButton label="2" isPressed={isPressed('Key2')} left={45} top={55} width={50} height={75} isBlackKey />
                        <KeyButton label="4" isPressed={isPressed('Key4')} left={115} top={55} width={50} height={75} isBlackKey />
                        <KeyButton label="6" isPressed={isPressed('Key6')} left={185} top={55} width={50} height={75} isBlackKey />

                        {/* White Keys (Bottom row of keyboard) */}
                        <KeyButton label="1" isPressed={isPressed('Key1')} left={10} top={135} width={50} height={75} />
                        <KeyButton label="3" isPressed={isPressed('Key3')} left={80} top={135} width={50} height={75} />
                        <KeyButton label="5" isPressed={isPressed('Key5')} left={150} top={135} width={50} height={75} />
                        <KeyButton label="7" isPressed={isPressed('Key7')} left={220} top={135} width={50} height={75} />
                    </Box>
                </Box>
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
