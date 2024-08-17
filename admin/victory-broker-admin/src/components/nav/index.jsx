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
} from "@mantine/core";
import {
  IconBellRinging,
  IconFingerprint,
  IconKey,
  IconSettings,
  Icon2fa,
  IconDatabaseImport,
  IconReceipt2,
  IconSwitchHorizontal,
  IconLogout,
  IconNetwork,
  IconDatabase,
  IconSearch,
  IconCloudNetwork,
} from "@tabler/icons-react";
import { MantineLogo } from "@mantinex/mantine-logo";
import classes from "./nav.module.css";

import { PubSubAdminServiceClient } from "admin-grpc-gen/Pubsub_adminServiceClientPb";
import * as AdminPB from "admin-grpc-gen/pubsub_admin_pb";
const data = [
  { link: "", label: "Adapters", icon: IconNetwork },
  { link: "", label: "Channels", icon: IconBellRinging },
  { link: "", label: "Data", icon: IconDatabase },
];

export function NavbarSimple() {
  const [active, setActive] = useState("Billing");
  const [client, setClient] = useState(null);

  useEffect(() => {
    console.log("Connecting to broker admin service");
    const client = new PubSubAdminServiceClient("http://0.0.0.0:5050");
    setClient(client);
  }, []);

  const links = data.map((item) => (
    <a
      className={classes.link}
      data-active={item.label === active || undefined}
      href={item.link}
      key={item.label}
      onClick={(event) => {
        event.preventDefault();
        setActive(item.label);
      }}
    >
      <item.icon className={classes.linkIcon} stroke={1.5} />
      <span>{item.label}</span>
    </a>
  ));

  useEffect(() => {
    if (active === "Channels") onChannels();
  }, [active]);

  const onChannels = () => {
    console.log("Channels");
    const request = new AdminPB.ChannelRequest();
    client.requestChannels(request).then((response) => {
      console.log(response);
    });
  };
  return (
    <nav className={classes.navbar}>
      <div className={classes.navbarMain}>
        <Group className={classes.header} justify="space-between">
          <h1>Victory Broker Admin</h1>
        </Group>
        {links}
      </div>

      <div className={classes.footer}>
        <TextInput
          placeholder="Broker Admin URL"
          size="xs"
          leftSection={
            <IconCloudNetwork
              style={{ width: rem(12), height: rem(12) }}
              stroke={1.5}
            />
          }
          rightSectionWidth={70}
          styles={{ section: { pointerEvents: "none" } }}
          mb="sm"
        />
        <a
          href="#"
          className={classes.link}
          onClick={(event) => event.preventDefault()}
        >
          <IconLogout className={classes.linkIcon} stroke={1.5} />
          <span>Connect</span>
        </a>
      </div>
    </nav>
  );
}
