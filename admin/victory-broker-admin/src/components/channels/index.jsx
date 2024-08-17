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

export function ChannelsTable() {

    const [channels, setChannels] = useState([]);
    useEffect(() => {
        console.log("Connecting to broker admin service");
        const client = new PubSubAdminServiceClient("http://0.0.0.0:5050");
        const request = new AdminPB.ChannelRequest();

        const stream = client.requestChannels(request);
        stream.on("data", (response) => {
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

        stream.on("end", () => {
            console.log("Stream ended");
        });

        stream.on("status", (status) => {   
            console.log("Stream status", status);
        });

    }, []);


    const card = (channel) => (
        <Card withBorder radius="md" padding="xl" bg="var(--mantine-color-body)" key={channel.topic}>
            <Text fz="xs" tt="uppercase" fw={700} c="dimmed">
                {channel.topic}
            </Text>
            <Text fz="lg" fw={500}>
                $5.431 / $10.000
            </Text>
            <Progress value={54.31} mt="md" size="lg" radius="xl" />
        </Card>
    );
    
    const renderCards = () => {
        return channels.map((channel) => {
            return card(channel);
        });
    }

    return (
        <SimpleGrid cols={3} >
            {renderCards()}
        </SimpleGrid>
    )
}
