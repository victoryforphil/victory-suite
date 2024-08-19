import "@mantine/core/styles.css";
import { AppShell, Burger, Container, Group, MantineProvider } from "@mantine/core";
import { theme } from "./theme";
import { NavbarSimple } from "./components/nav";
import { ChannelsTable } from "./components/channels";
import { useDisclosure } from '@mantine/hooks';
import { MantineLogo } from "@mantinex/mantine-logo";
import { useState , useEffect} from "react";
import { PubSubAdminServiceClient } from "admin-grpc-gen/Pubsub_adminServiceClientPb";

export default function App() {
  const [opened, { toggle }] = useDisclosure();
  const [grpcClient, setGrpcClient] = useState(null);

  const [grpcState, setGrpcState] = useState(0);
  const [channelState, setChannelState] = useState(0);
  const [allStates, setAllStates] = useState({loading: false, connected: false, statuses: []});

  const connect = (url) => {
    setAllStates({loading: true, connected: false, statuses: []});
    console.log("Connecting to broker admin service: " + url);
    const client = new PubSubAdminServiceClient(url);

    setGrpcClient(client);
  };
  
  useEffect(() => {
    
      let statses = {
        "Channels": channelState,
       
      };

      if (channelState == 2 ){
        setAllStates({loading: false, connected: true, statuses: statses});
      }else{
        setAllStates({loading: allStates.loading, connected: allStates.connected, statuses: statses});

      }

  }, [channelState]);

  return (
    <MantineProvider theme={theme} defaultColorScheme="dark">>
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
        <NavbarSimple onConnect={connect} states={allStates}  />
        </AppShell.Navbar>
        <AppShell.Main> 

          <Container>
          <ChannelsTable grpc={grpcClient} onStateUpdate ={(code) =>{
            console.log("State update: " + code);
              setChannelState(code);
          
            }}/>
          </Container>
        </AppShell.Main>
      </AppShell>
     
     
    </MantineProvider>
  );
}
