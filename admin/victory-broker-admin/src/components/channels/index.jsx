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

const data = [

];

let doOnce = true;
export function ChannelsTable() {

    const [channels, setChannels] = useState([]);
    const [stream, setStream] = useState(null);
    useEffect(() => {

        if (!doOnce) {
            return;
        }

        doOnce = false;
        console.log("Connecting to broker admin service");
        const client = new PubSubAdminServiceClient("http://0.0.0.0:5050");
        const request = new AdminPB.ChannelRequest();

        const steam = client.requestChannels(request);
      
        steam.on("data", (response) => {
            console.log("Received data");
            const channels = response.getChannelsList().map((channel) => {
                console.log(channel);
                return {
                    topic: channel.getTopic(),
                    publishers: channel.getPublishersList(),
                    subscribers: channel.getSubscribersList(),
                    message_count: channel.getMessageCount(),
                };
            });
            setChannels(channels);
            console.dir(channels)
        });

        

    }, []);


    const row = (channel) => (
        <Table.Tr key={channel.topic}>
            <Table.Td><Code>{channel.topic}</Code></Table.Td>
            <Table.Td>{channel.publishers.join(", ")}</Table.Td>
            <Table.Td>{channel.subscribers.join(", ")}</Table.Td>
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
