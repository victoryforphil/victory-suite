import "@mantine/core/styles.css";
import { MantineProvider } from "@mantine/core";
import { theme } from "./theme";
import { NavbarSimple } from "./components/nav";

export default function App() {
  return (
    <MantineProvider theme={theme}>
      <NavbarSimple />
    </MantineProvider>
  );
}
