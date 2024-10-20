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



export function AdaptersTable({ grpc, onStateUpdate }) {

    const [adapters, setAdapters] = useState([]);
    const [stream, setStream] = useState(null);
    useEffect(() => {

        if (grpc == null) {
            onStateUpdate(0);
            return;
        }
        onStateUpdate(1);

        console.log("Connecting to broker admin service");

        const request = new AdminPB.AdapterRequest();

        const steam = grpc.requestAdapters(request);


        steam.on("data", (response) => {
            onStateUpdate(2);
            const adapters = response.getAdaptersList().map((adapter) => {
                return {
                    name: adapter.getName(),
                    description: adapter.getDescription(),
                    live: adapter.getLive(),
                    stats: adapter.getStatsList(),
                };

            });
            setAdapters(adapters);

        });

    }, [grpc]);


    const row = (adapter) => (
        <Table.Tr key={adapter.name}>
            <Table.Td>{adapter.name}</Table.Td>
            <Table.Td><Code>{adapter.description}</Code></Table.Td>
            <Table.Td>{adapter.stats.map((stat) => {
                return (
                    <Badge color="blue"> {stat}</Badge>
                );
            }
            )}</Table.Td>
            <Table.Td>{adapter.live ? <Badge color="green" variant="dot">Live</Badge> : <Badge color="red" variant="dot">Not Live</Badge>}</Table.Td>


      </Table.Tr>
    );

    const rows = adapters.map(row);

    return (
        <Table>
      <Table.Thead>
        <Table.Tr>
            <Table.Th>name</Table.Th>
            <Table.Th>description</Table.Th>
            <Table.Th>stats</Table.Th>
            <Table.Th>live</Table.Th>

        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>{rows}</Table.Tbody>
    </Table>
    )
}
