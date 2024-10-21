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
  Card,
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
import { useNavigate } from "react-router-dom";
const data = [
  { link : "/adapters", label: "Adapters", icon: IconNetwork },
  { link: "/channels", label: "Channels", icon: IconBellRinging },
  { link: "", label: "Data", icon: IconDatabase },
];

export function NavbarSimple({ onConnect, states }) {
  const [active, setActive] = useState("Billing");
  const [url, setUrl] = useState("http://localhost:5050");


  const links = data.map((item) => (
    <a
      className={classes.link}
      data-active={item.label === active || undefined}
      href={item.link}
      key={item.label}
      onClick={(event) => {
      //  event.preventDefault();
        setActive(item.label);


      }}
    >
      <item.icon className={classes.linkIcon} stroke={1.5} />
      <span>{item.label}</span>
      <hr />
      {states.statuses[item.label] == 2 ? <Badge color="green" variant="dot">Connected</Badge> : <Badge color="red" variant="dot">Disconnected</Badge>}
    </a>
  ));



  const onConnectClick = (event) => {
    event.preventDefault();
    console.log("Connecting to broker admin service: " + url);
    onConnect(url);
  }



  return (
    <nav className={classes.navbar}>
      <div className={classes.navbarMain}>
        {links}
      </div>

      <div className={classes.footer}>
        <TextInput
          placeholder="Broker Admin URL"
          size="xs"
          defaultValue={url}

          onChange={(event) => setUrl(event.target.value)}
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
          onClick={onConnectClick}
          data-active={states.loading || undefined}
        >
          <IconLogout className={classes.linkIcon} stroke={1.5} />
          <span>{states.loading ? "Connecting..." : states.connected ? "Disconnect" : "Connect"}</span>
        </a>


      </div>
    </nav>
  );
}
