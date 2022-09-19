#!/usr/bin/env bash
# Thanks https://raw.githubusercontent.com/ernisn/superspeed/master/superspeed.sh

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE="\033[0;35m"
CYAN='\033[0;36m'
PLAIN='\033[0m'

check_wget() {
	if  [ ! -e '/usr/bin/wget' ]; then
	    echo "请先安装 wget" && exit 1
	fi
}

check_bimc() {
	if  [ ! -e './bimc' ]; then
		echo "正在获取 bim-core"
		wget --no-check-certificate -qO bimc https://github.com/veoco/bim-core/releases/download/v0.7.6/bimc-v0.7.6-$(uname -m)-unknown-linux-musl > /dev/null 2>&1
        chmod +x bimc
	fi
}

print_info() {
	echo "—————————————————————————— HyperSpeed ——————————————————————————————"
	echo "         bash <(curl -Lso- https://git.io/superspeed)"
	echo "         节点更新: 2022/09/19  | 脚本更新: 2022/09/19"
	echo "————————————————————————————————————————————————————————————————————"
}



get_options() {
	echo -e "  测速类型:    ${GREEN}1.${PLAIN} 三网测速    ${GREEN}2.${PLAIN} 取消测速"
	echo -e "               ${GREEN}3.${PLAIN} 电信节点    ${GREEN}4.${PLAIN} 联通节点    ${GREEN}5.${PLAIN} 移动节点"
	while :; do read -p "  请选择测速类型: " selection
			if [[ ! $selection =~ ^[1-5]$ ]]; then
					echo -e "  ${RED}输入错误${PLAIN}, 请输入正确的数字!"
			else
					break   
			fi
	done
    while :; do read -p "  请输入测速线程数量: " thread
			if [[ ! $thread =~ ^[1-9]$ ]]; then
					echo -e "  ${RED}输入错误${PLAIN}, 请输入正确的数字!"
			else
					break   
			fi
	done
}


speed_test(){
    local nodeID=$1
	local nodeLocation=$2
	local nodeISP=$3

    strnodeLocation="${nodeLocation}　　　　　　"
	LANG=C
	#echo $LANG

    printf "\r${RED}%-6s${YELLOW}%s%s${GREEN}%-24s${CYAN}%s%-10s${BLUE}%s%-10s${GREEN}%-10s${PURPLE}%-6s${PLAIN}" "${nodeID}"  "${nodeISP}" "|" "${strnodeLocation:0:24}" "↑ " "..." "↓ " "..." "..." "..."

	output=$(./bimc $1 -t $thread)

	local upload=$(echo $output | cut -d ',' -f1)
    local download=$(echo $output | cut -d ',' -f2)
	local latency=$(echo $output | cut -d ',' -f3)
    local jitter=$(echo $output | cut -d ',' -f4)
			
	if [[ $(awk -v num1=${upload} -v num2=0.0 'BEGIN{print(num1>num2)?"1":"0"}') -eq 1 ]]; then
	    printf "\r${RED}%-6s${YELLOW}%s%s${GREEN}%-24s${CYAN}%s%-10s${BLUE}%s%-10s${GREEN}%-10s${PURPLE}%-6s${PLAIN}\n" "${nodeID}"  "${nodeISP}" "|" "${strnodeLocation:0:24}" "↑ " "${upload}" "↓ " "${download}" "${latency}" "${jitter}"
	fi

}

run_test() {
	[[ ${selection} == 2 ]] && exit 1

    echo "————————————————————————————————————————————————————————————————————"
	echo "ID    测速服务器信息       上传/Mbps   下载/Mbps   延迟/ms   抖动/ms"
	start=$(date +%s) 
	if [[ ${selection} == 1 ]]; then
		speed_test '595' '上海' '电信'
	fi

	if [[ ${selection} == 3 ]]; then
        speed_test '595' '上海' '电信'
	fi

	if [[ ${selection} == 4 ]]; then
        speed_test '595' '上海' '电信'
	fi

	if [[ ${selection} == 5 ]]; then
        speed_test '595' '上海' '电信'
	fi

    end=$(date +%s)
		echo -e "\r————————————————————————————————————————————————————————————————————"
		time=$(( $end - $start ))
		if [[ $time -gt 60 ]]; then
			min=$(expr $time / 60)
			sec=$(expr $time % 60)
			echo -ne "  测试完成, 本次测速耗时: ${min} 分 ${sec} 秒"
		else
			echo -ne "  测试完成, 本次测速耗时: ${time} 秒"
		fi
		echo -ne "\n  当前时间: "
		echo $(date +%Y-%m-%d" "%H:%M:%S)
}

run_all() {
	check_wget;
	check_bimc;
	clear
	print_info;
	get_options;
	run_test;
	rm -rf bimc
}

run_all
