import { Native } from "@withfig/api-bindings";

export default function ExternalLink({
  href,
  onClick,
  ...props
}: { href: string } & React.HTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      {...props}
      onClick={(e) => {
        Native.open(href).catch(console.error);
        onClick?.(e);
      }}
    />
  );
}
