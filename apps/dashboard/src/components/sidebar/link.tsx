import { cn } from "@/lib/utils";
import { Sparkle } from "../svg/icons"
import { NavLink } from "react-router-dom";

const defaultStyles = "px-3 py-1.5 h-10 hover:bg-[#6E3BF1]/70 text-zinc-600 dark:text-zinc-400 hover:text-white dark:hover:text-white transition-colors w-full rounded-lg flex flex-row items-center font-light"
const activeStyles = cn(defaultStyles, "bg-[#6E3BF1] hover:bg-[#6E3BF1]/90 text-white dark:text-white" )

export default function SidebarLink({
    icon = <Sparkle />,
    name,
    path = '/',
    count
}: {
    icon?: React.ReactNode,
    name: string,
    path?: string,
    count?: number
}) {

    return (
        <NavLink to={path} className={({isActive}) => isActive ? activeStyles : defaultStyles }>
            <div className="flex flex-row items-center gap-2">
                {icon}
                <span>{name}</span>
            </div>
            {count && count > 0 && (
                <div className="flex items-center justify-center p-2 px-3 rounded-md">
                    {count > 9 ? "9+" : count}
                </div>
            )}
        </NavLink>
    )
}