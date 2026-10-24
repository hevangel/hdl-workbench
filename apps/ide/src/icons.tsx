import type { SVGProps } from "react";

// Small stroke icons drawn on a 16px grid; they inherit the button's text colour.
type IconProps = SVGProps<SVGSVGElement>;

function Icon({ children, ...props }: IconProps) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false" {...props}>
      {children}
    </svg>
  );
}

export const SidebarIcon = (props: IconProps) => <Icon {...props}><rect x="2" y="2.5" width="12" height="11" rx="1.5" /><path d="M6 2.5v11" /></Icon>;
export const CollapseLeftIcon = (props: IconProps) => <Icon {...props}><path d="M9.5 4 5.5 8l4 4" /><path d="M12.5 4v8" /></Icon>;
export const PlayIcon = (props: IconProps) => <Icon {...props}><path d="M5 3.5v9l7-4.5z" fill="currentColor" stroke="none" /></Icon>;
export const SpinnerIcon = (props: IconProps) => <Icon {...props} className="spin"><path d="M8 2a6 6 0 1 0 6 6" /></Icon>;
export const SaveIcon = (props: IconProps) => <Icon {...props}><path d="M3 3.5A1.5 1.5 0 0 1 4.5 2h6L13 4.5v8A1.5 1.5 0 0 1 11.5 14h-7A1.5 1.5 0 0 1 3 12.5z" /><path d="M5.5 2v3h4V2M5 14V9.5h6V14" /></Icon>;
export const ImportIcon = (props: IconProps) => <Icon {...props}><path d="M8 2.5v7M5 6.5l3 3 3-3" /><path d="M3 10.5v2A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-2" /></Icon>;
export const ExportIcon = (props: IconProps) => <Icon {...props}><path d="M8 9.5v-7M5 5.5l3-3 3 3" /><path d="M3 10.5v2A1.5 1.5 0 0 0 4.5 14h7a1.5 1.5 0 0 0 1.5-1.5v-2" /></Icon>;
export const AgentIcon = (props: IconProps) => <Icon {...props}><path d="M6 5 3 8l3 3M10 5l3 3-3 3" /></Icon>;
export const WarningIcon = (props: IconProps) => <Icon {...props}><path d="M8 2.5 14 13H2z" /><path d="M8 6.5v3M8 11.2v.1" /></Icon>;
export const ChevronIcon = (props: IconProps) => <Icon {...props}><path d="m5.5 6.5 2.5 2.5 2.5-2.5" /></Icon>;
export const FolderIcon = (props: IconProps) => <Icon {...props}><path d="M2 4.5A1.5 1.5 0 0 1 3.5 3h3l1.5 1.5h4.5A1.5 1.5 0 0 1 14 6v5.5a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 11.5z" /></Icon>;
export const HelpIcon = (props: IconProps) => <Icon {...props}><circle cx="8" cy="8" r="6" /><path d="M6.4 6.3a1.7 1.7 0 1 1 2.3 1.6c-.45.2-.7.55-.7 1v.35" /><path d="M8 11.3v.1" /></Icon>;
export const MoreIcon = (props: IconProps) => <Icon {...props}><circle cx="4" cy="8" r=".6" fill="currentColor" /><circle cx="8" cy="8" r=".6" fill="currentColor" /><circle cx="12" cy="8" r=".6" fill="currentColor" /></Icon>;

/** Brand mark: one clock period as a digital waveform. */
export const BrandMark = (props: IconProps) => (
  <Icon viewBox="0 0 20 20" width="20" height="20" strokeWidth="1.6" {...props}>
    <path d="M2 13h3.5V7H10v6h3.5V7H18" />
  </Icon>
);
