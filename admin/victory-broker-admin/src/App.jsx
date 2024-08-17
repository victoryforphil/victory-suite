import "@mantine/core/styles.css";
import { AppShell, Burger, Container, Group, MantineProvider } from "@mantine/core";
import { theme } from "./theme";
import { NavbarSimple } from "./components/nav";
import { ChannelsTable } from "./components/channels";
import { useDisclosure } from '@mantine/hooks';
import { MantineLogo } from "@mantinex/mantine-logo";
export default function App() {
  const [opened, { toggle }] = useDisclosure();
  return (
    <MantineProvider theme={theme}>
      <AppShell
        header={{ height: 60 }}
        navbar={{ width: 300, breakpoint: 'sm', collapsed: { mobile: !opened } }}
        padding="md"
      >
        <AppShell.Header>
          <Group h="100%" px="md">
            <Burger opened={opened} onClick={toggle} hiddenFrom="sm" size="sm" />
            <h3>Victory Broker Admin</h3>
          </Group>
        </AppShell.Header>
        <AppShell.Navbar p="md">
        <NavbarSimple />
        </AppShell.Navbar>
        <AppShell.Main> 

          <Container>
          <ChannelsTable />
          </Container>
        </AppShell.Main>
      </AppShell>
     
     
    </MantineProvider>
  );
}
