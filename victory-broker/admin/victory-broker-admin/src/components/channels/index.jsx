import { useState, useEffect } from "react";
import {
    TextInput,
    Code,
    UnstyledButton,
    Badge,
    Text,
    Group,
    ActionIcon,
    Tooltip,
    rem,
    Table,
    Card,
    SimpleGrid,
    Progress,
} from "@mantine/core";
import {
    IconBellRinging,

    IconLogout,
    IconNetwork,
    IconDatabase,
    IconSearch,
    IconCloudNetwork,
} from "@tabler/icons-react";


import { PubSubAdminServiceClient } from "admin-grpc-gen/Pubsub_adminServiceClientPb";
import * as AdminPB from "admin-grpc-gen/pubsub_admin_pb";



export function ChannelsTable({ grpc, onStateUpdate }) {

    const [channels, setChannels] = useState([]);
    const [stream, setStream] = useState(null);
    useEffect(() => {

        if (grpc == null) {
            onStateUpdate(0);
            return;
        }
        onStateUpdate(1);
       
        console.log("Connecting to broker admin service");
       
        const request = new AdminPB.ChannelRequest();

        const steam = grpc.requestChannels(request);
            
        
        steam.on("data", (response) => {
            onStateUpdate(2);
            const channels = response.getChannelsList().map((channel) => {
                
                return {
                    topic: channel.getTopic(),
                    publishers: channel.getPublishersList(),
                    subscribers: channel.getSubscribersList(),
                    message_count: channel.getMessageCount(),
                };
            });
            setChannels(channels);
          
        });
     

      

        

    }, [grpc]);


    const row = (channel) => (
        <Table.Tr key={channel.topic}>
            <Table.Td><Code>{channel.topic}</Code></Table.Td>
            <Table.Td>{channel.publishers.map(v => <Badge color="orange">{v}</Badge>)}</Table.Td>
            <Table.Td>{channel.subscribers.map(v => <Badge color="orange">{v}</Badge>)}</Table.Td>
            <Table.Td><Progress value={channel.message_count}></Progress></Table.Td>
      </Table.Tr>
    );
    
    const rows = channels.map(row);

    return (
        <Table>
      <Table.Thead>
        <Table.Tr>
            <Table.Th>Topic</Table.Th>
            <Table.Th>Publishers</Table.Th>
            <Table.Th>Subscribers</Table.Th>
            <Table.Th>Message Count</Table.Th>
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>{rows}</Table.Tbody>
    </Table>
    )
}
