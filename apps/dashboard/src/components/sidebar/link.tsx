import { cn } from "@/lib/utils";
import { NavLink } from "react-router-dom";

const defaultStyles = "p-2 h-10 hover:bg-[#6E3BF1]/10 text-zinc-500 dark:text-zinc-400 dark:hover:text-white transition-colors w-full rounded-lg flex flex-row items-center justify-between font-light [&>span]:bg-zinc-500 [&>span]:text-white hover:text-[#6E3BF1] [&:hover>span]:bg-[#6E3BF1]"
const activeStyles = cn(defaultStyles, "bg-[#6E3BF1] hover:bg-[#6E3BF1] text-white dark:text-white [&>span]:bg-white [&>span]:text-[#6E3BF1] [&:hover>span]:text-[#6E3BF1] [&:hover>span]:bg-white hover:text-white" )

export default function SidebarLink({
    icon,
    name,
    path = '/',
    count
}: {
    icon: React.ReactNode,
    name: string,
    path?: string,
    count?: number
}) {

    return (
        <NavLink to={path} className={({isActive}) => isActive ? activeStyles : defaultStyles }>
            <div className="flex flex-row items-center gap-2 fill-current whitespace-nowrap">
                {icon}
                <span className="text-base">{name}</span>
            </div>
            {count && (
                <span className="flex items-center justify-center leading-none p-1 px-2 rounded-full font-medium text-sm">
                    {count > 9 ? "9+" : count}
                </span>
            )}
        </NavLink>
    )
}