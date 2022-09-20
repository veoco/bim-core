#!/usr/bin/env bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE="\033[0;35m"
CYAN='\033[0;36m'
ENDC='\033[0m'

check_wget() {
    if  [ ! -e '/usr/bin/wget' ]; then
        echo "请先安装 wget" && exit 1
    fi
}

check_bimc() {
    if  [ ! -e './bimc' ]; then
        echo "正在获取组件"
        arch=$(uname -m)
        if [ "${arch}" == "i686" ]; then
            arch="i586"
        fi
        wget --no-check-certificate -qO bimc https://bench.im/bimc-$(arch) > /dev/null 2>&1
        chmod +x bimc
    fi
}

print_info() {
    echo "—————————————————————————— HyperSpeed ———————————————————————————————"
    echo "          bash <(curl -Lso- https://bench.im/hyperspeed)"
    echo "          项目修改自: https://github.com/zq/superspeed"
    echo "     节点更新: 2022/08/20 | 脚本更新: 2022/09/20 | 组件版本: 0.7.7"
    echo "—————————————————————————————————————————————————————————————————————"
}



get_options() {
    echo -e "  测速类型:    ${GREEN}1.${ENDC} 三网测速    ${GREEN}2.${ENDC} 取消测速    ${GREEN}0.${ENDC} 港澳台日韩"
    echo -e "               ${GREEN}3.${ENDC} 电信节点    ${GREEN}4.${ENDC} 联通节点    ${GREEN}5.${ENDC} 移动节点"
    while :; do read -p "  请选择测速类型: " selection
            if [[ ! $selection =~ ^[0-5]$ ]]; then
                    echo -e "  ${RED}输入错误${ENDC}, 请输入正确的数字!"
            else
                    break   
            fi
    done
    while :; do read -p "  请输入测速线程数量: " thread
            if [[ ! $thread =~ ^[1-9][0-9]?$ ]]; then
                    echo -e "  ${RED}输入错误${ENDC}, 请输入正确的数字!"
            else
                    break   
            fi
    done
}


speed_test(){
    local nodeID=$1
    local nodeLocation=$2
    local nodeISP=$3

    local name=$(./bimc 0 -n "$nodeLocation")

    printf "\r${RED}b%-6s${YELLOW}%s%s${GREEN}%s${CYAN}%s%-10s${BLUE}%s%-10s${GREEN}%-10s${PURPLE}%-6s${ENDC}" "${nodeID}"  "${nodeISP}" "|" "${name}" "↑ " "..." "↓ " "..." "..." "..."

    output=$(./bimc $1 -t $thread)

    local upload=$(echo $output | cut -d ',' -f1)
    local download=$(echo $output | cut -d ',' -f2)
    local latency=$(echo $output | cut -d ',' -f3)
    local jitter=$(echo $output | cut -d ',' -f4)
            
    if [[ $(awk -v num1=${upload} -v num2=0.0 'BEGIN{print(num1>num2)?"1":"0"}') -eq 1 ]]; then
        printf "\r${RED}b%-6s${YELLOW}%s%s${GREEN}%s${CYAN}%s%-10s${BLUE}%s%-10s${GREEN}%-10s${PURPLE}%-6s${ENDC}\n" "${nodeID}"  "${nodeISP}" "|" "${name}" "↑ " "${upload}" "↓ " "${download}" "${latency}" "${jitter}"
    fi

}

run_test() {
    [[ ${selection} == 2 ]] && exit 1

    echo "—————————————————————————————————————————————————————————————————————"
    echo "ID     测速服务器信息       上传/Mbps   下载/Mbps   延迟/ms   抖动/ms"
    start=$(date +%s) 

    if [[ ${selection} == 1 ]] || [[ ${selection} == 3 ]]; then
        speed_test '595' '上海' '电信'
        speed_test '5641' '江苏南京5G' '电信'
        speed_test '6340' '四川成都' '电信'
    fi

    if [[ ${selection} == 1 ]] || [[ ${selection} == 4 ]]; then
        speed_test '5135' '上海5G' '联通'
        speed_test '868' '湖南长沙5G' '联通'
        speed_test '8881' '辽宁沈阳' '联通'
    fi

    if [[ ${selection} == 1 ]] || [[ ${selection} == 5 ]]; then
        speed_test '1332' '浙江宁波' '移动'
        speed_test '3430' '福建福州' '移动'
        speed_test '817' '四川成都' '移动'
    fi

    if [[ ${selection} == 0 ]]; then
        speed_test '12715' '香港宽频' '香港'
        speed_test '7554' '澳门电讯' '澳门'
        speed_test '3903' '中华电信' '台北'
        speed_test '5109' '乐天移动' '东京'
        speed_test '1294' 'Kdatacenter ' '首尔'
    fi

    end=$(date +%s)
        echo -e "\r—————————————————————————————————————————————————————————————————————"
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

LANG=C
run_all
