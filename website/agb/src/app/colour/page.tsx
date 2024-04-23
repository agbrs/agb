import { Metadata } from "next";
import ColourPicker from "./colour";

export const metadata: Metadata = {
  title: "Colour Converter",
};

export default function ColourPickerPage() {
  return <ColourPicker />;
}
