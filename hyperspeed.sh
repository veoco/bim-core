#!/usr/bin/env bash

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
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
    echo "——————————————————————————— HyperSpeed ———————————————————————————————"
    echo "           bash <(wget -qO- https://bench.im/hyperspeed)"
    echo "           项目修改自: https://github.com/zq/superspeed/"
    echo "      节点更新: 2022/11/15 | 脚本更新: 2022/11/15 | 组件版本: 0.9.2"
    echo "——————————————————————————————————————————————————————————————————————"
}



get_options() {
    echo -e "  测速类型:    ${GREEN}1.${ENDC} 三网测速    ${GREEN}2.${ENDC} 取消测速    ${GREEN}0.${ENDC} 港澳台日韩"
    echo -e "               ${GREEN}3.${ENDC} 电信节点    ${GREEN}4.${ENDC} 联通节点    ${GREEN}5.${ENDC} 移动节点"
    echo -e "               ${GREEN}6.${ENDC} 教育网IPv4  ${GREEN}7.${ENDC} 教育网IPv6"
    while :; do read -p "  请选择测速类型(默认: 1): " selection
        if [[ "$selection" == "" ]]; then
            selection=1
            break
        elif [[ ! $selection =~ ^[0-7]$ ]]; then
            echo -e "  ${RED}输入错误${ENDC}, 请输入正确的数字!"
        else
            break   
        fi
    done
    while :; do read -p "  启用多线程测速(留空禁用): " multi
        if [[ "$multi" != "" ]]; then
            thread=" -m"
            break
        else
            thread=""
            break 
        fi
    done
}


speed_test(){
    local nodeType=$1
    local nodeLocation=$2
    local nodeISP=$3
    local extra=$4
    local dl=$(echo "$5"| base64 -d)
    local ul=$(echo "$6"| base64 -d)
    local name=$(./bimc 0 0 -n "$nodeLocation")

    printf "\r${GREEN}%-7s${YELLOW}%s%s${GREEN}%s${CYAN}%s%-11s${CYAN}%s%-11s${GREEN}%-9s${PURPLE}%-7s${ENDC}" "${nodeType}"  "${nodeISP}" "|" "${name}" "↑ " "..." "↓ " "..." "..." "..."

    output=$(./bimc $dl $ul$thread $extra)
    local upload="$(echo "$output" | cut -n -d ',' -f1)"
    local download="$(echo "$output" | cut -n -d ',' -f2)"
    local latency="$(echo "$output" | cut -n -d ',' -f3)"
    local jitter="$(echo "$output" | cut -n -d ',' -f4)"
            
    printf "\r${GREEN}%-7s${YELLOW}%s%s${GREEN}%s${CYAN}%s%s${CYAN}%s%s${GREEN}%s${PURPLE}%s${ENDC}\n" "${nodeType}"  "${nodeISP}" "|" "${name}" "↑ " "${upload}" "↓ " "${download}" "${latency}" "${jitter}"
}

run_test() {
    [[ ${selection} == 2 ]] && exit 1

    echo "——————————————————————————————————————————————————————————————————————"
    echo "协议   测速服务器信息       上传/Mbps    下载/Mbps    延迟/ms  抖动/ms"
    start=$(date +%s) 

    if [[ ${selection} == 1 ]] || [[ ${selection} == 3 ]]; then
        speed_test 'HTTP' '上海' '电信' '' 'aHR0cDovL3NwZWVkdGVzdDEub25saW5lLnNoLmNuOjgwODAvZG93bmxvYWQK' 'aHR0cDovL3NwZWVkdGVzdDEub25saW5lLnNoLmNuOjgwODAvdXBsb2FkCg=='
        speed_test 'HTTP' '天津5G' '电信' '' 'aHR0cDovL3N5LnRqdGVsZS5jb206ODA4MC9kb3dubG9hZA==' 'aHR0cDovL3N5LnRqdGVsZS5jb206ODA4MC91cGxvYWQ='
        speed_test 'HTTP' '重庆5G' '电信' '' 'aHR0cDovL3NwZWVkLmNxdGVsZWNvbS5jb20uY246ODA4MC9kb3dubG9hZA==' 'aHR0cDovL3NwZWVkLmNxdGVsZWNvbS5jb20uY246ODA4MC91cGxvYWQ='
        speed_test 'HTTP' '湖北武汉' '电信' '' 'aHR0cDovL3ZpcHNwZWVkdGVzdDEud3VoYW4ubmV0LmNuOjgwODAvZG93bmxvYWQ=' 'aHR0cDovL3ZpcHNwZWVkdGVzdDEud3VoYW4ubmV0LmNuOjgwODAvdXBsb2Fk'
        speed_test 'HTTP' '江苏南京5G' '电信' '' 'aHR0cDovLzVnbmFuamluZy5zcGVlZHRlc3QuanNpbmZvLm5ldDo4MDgwL2Rvd25sb2FkCg==' 'aHR0cDovLzVnbmFuamluZy5zcGVlZHRlc3QuanNpbmZvLm5ldDo4MDgwL3VwbG9hZAo='
    fi

    if [[ ${selection} == 1 ]] || [[ ${selection} == 4 ]]; then
        speed_test 'HTTP' '上海5G' '联通' '' 'aHR0cDovLzVnLnNodW5pY29tdGVzdC5jb206ODA4MC9kb3dubG9hZAo=' 'aHR0cDovLzVnLnNodW5pY29tdGVzdC5jb206ODA4MC91cGxvYWQK'
        speed_test 'HTTP' '河南郑州5G' '联通' '' 'aHR0cDovLzVndGVzdC5zaGFuZ2R1LmNvbTo4MDgwL2Rvd25sb2Fk' 'aHR0cDovLzVndGVzdC5zaGFuZ2R1LmNvbTo4MDgwL3VwbG9hZA=='
        speed_test 'HTTP' '湖南长沙5G' '联通' '' 'aHR0cDovL3NwZWVkdGVzdDAxLmhuMTY1LmNvbTo4MDgwL2Rvd25sb2Fk' 'aHR0cDovL3NwZWVkdGVzdDAxLmhuMTY1LmNvbTo4MDgwL3VwbG9hZA=='
        speed_test 'HTTP' '辽宁沈阳' '联通' '' 'aHR0cDovL3VuaWNvbXNwZWVkdGVzdC5jb206ODA4MC9kb3dubG9hZAo=' 'aHR0cDovL3VuaWNvbXNwZWVkdGVzdC5jb206ODA4MC91cGxvYWQK'
        speed_test 'HTTPS' '江苏无锡' '联通' '' 'aHR0cHM6Ly9zcGVlZHRlc3QyLm5pdXRrLmNvbTo4MDgwL2Rvd25sb2Fk' 'aHR0cHM6Ly9zcGVlZHRlc3QyLm5pdXRrLmNvbTo4MDgwL3VwbG9hZA=='
    fi

    if [[ ${selection} == 1 ]] || [[ ${selection} == 5 ]]; then
        speed_test 'HTTP' '北京' '移动' '' 'aHR0cDovLzIxMS4xMzYuMzAuMTE0OjkwMDAvc3BlZWQvMjAwMDAwMC5kYXRhCg==' 'aHR0cDovLzIxMS4xMzYuMzAuMTE0OjkwMDAvc3BlZWQvMjAwMDAwLmRhdGEK'
        speed_test 'HTTP' '河南郑州5G' '移动' '' 'aHR0cDovL3NwZWVkdGVzdC5lYXN0Y29tLnNpdGU6ODA4MC9kb3dubG9hZA==' 'aHR0cDovL3NwZWVkdGVzdC5lYXN0Y29tLnNpdGU6ODA4MC91cGxvYWQ='
        speed_test 'HTTP' '陕西西安5G' '移动' '' 'aHR0cDovL3NwZWVkdGVzdC5vbmUtcHVuY2gud2luOjgwODAvZG93bmxvYWQ=' 'aHR0cDovL3NwZWVkdGVzdC5vbmUtcHVuY2gud2luOjgwODAvdXBsb2Fk'
        speed_test 'HTTP' '四川成都' '移动' '' 'aHR0cDovL3NwZWVkdGVzdDEuc2MuY2hpbmFtb2JpbGUuY29tOjgwODAvZG93bmxvYWQ=' 'aHR0cDovL3NwZWVkdGVzdDEuc2MuY2hpbmFtb2JpbGUuY29tOjgwODAvdXBsb2Fk'
        speed_test 'HTTP' '甘肃兰州' '移动' '' 'aHR0cDovL3NwZWVkdGVzdDEuZ3MuY2hpbmFtb2JpbGUuY29tOjgwODAvZG93bmxvYWQ=' 'aHR0cDovL3NwZWVkdGVzdDEuZ3MuY2hpbmFtb2JpbGUuY29tOjgwODAvdXBsb2Fk'
    fi

    if [[ ${selection} == 6 ]]; then
        speed_test 'HTTPS' '中国科技大学' '合肥' '' 'aHR0cHM6Ly90ZXN0LnVzdGMuZWR1LmNuL2JhY2tlbmQvZ2FyYmFnZS5waHAK' 'aHR0cHM6Ly90ZXN0LnVzdGMuZWR1LmNuL2JhY2tlbmQvZW1wdHkucGhwCg=='
        speed_test 'HTTPS' '东北大学' '沈阳' '' 'aHR0cHM6Ly9pcHR2LnRzaW5naHVhLmVkdS5jbi9zdC9nYXJiYWdlLnBocAo=' 'aHR0cHM6Ly9pcHR2LnRzaW5naHVhLmVkdS5jbi9zdC9lbXB0eS5waHAK'
        speed_test 'HTTPS' '上海交通大学' '上海' '' 'aHR0cHM6Ly93c3VzLnNqdHUuZWR1LmNuL3NwZWVkdGVzdC9iYWNrZW5kL2dhcmJhZ2UucGhwCg==' 'aHR0cHM6Ly93c3VzLnNqdHUuZWR1LmNuL3NwZWVkdGVzdC9iYWNrZW5kL2VtcHR5LnBocAo='
    fi

    if [[ ${selection} == 7 ]]; then
        speed_test 'HTTPS' '中国科技大学' '合肥' '-6' 'aHR0cHM6Ly90ZXN0Ni51c3RjLmVkdS5jbi9iYWNrZW5kL2dhcmJhZ2UucGhwCg==' 'aHR0cHM6Ly90ZXN0Ni51c3RjLmVkdS5jbi9iYWNrZW5kL2VtcHR5LnBocAo='
        speed_test 'HTTPS' '东北大学' '沈阳' '-6' 'aHR0cHM6Ly9pcHR2LnRzaW5naHVhLmVkdS5jbi9zdC9nYXJiYWdlLnBocAo=' 'aHR0cHM6Ly9pcHR2LnRzaW5naHVhLmVkdS5jbi9zdC9lbXB0eS5waHAK'
        speed_test 'HTTPS' '上海交通大学' '上海' '-6' 'aHR0cHM6Ly93c3VzLnNqdHUuZWR1LmNuL3NwZWVkdGVzdC9iYWNrZW5kL2dhcmJhZ2UucGhwCg==' 'aHR0cHM6Ly93c3VzLnNqdHUuZWR1LmNuL3NwZWVkdGVzdC9iYWNrZW5kL2VtcHR5LnBocAo='
    fi

    if [[ ${selection} == 0 ]]; then
        speed_test 'HTTP' '环电宽频' '香港' '' 'aHR0cDovL29va2xhLWhpZGMuaGdjb25haXIuaGdjLmNvbS5oazo4MDgwL2Rvd25sb2FkCg==' 'aHR0cDovL29va2xhLWhpZGMuaGdjb25haXIuaGdjLmNvbS5oazo4MDgwL3VwbG9hZAo='
        speed_test 'HTTP' '澳门电讯' '澳门' '' 'aHR0cDovL3NwZWVkdGVzdDUubWFjYXUuY3RtLm5ldDo4MDgwL2Rvd25sb2FkCg==' 'aHR0cDovL3NwZWVkdGVzdDUubWFjYXUuY3RtLm5ldDo4MDgwL3VwbG9hZAo='
        speed_test 'HTTP' '中华电信' '台北' '' 'aHR0cDovL3RwMS5jaHRtLmhpbmV0Lm5ldDo4MDgwL2Rvd25sb2FkCg==' 'aHR0cDovL3RwMS5jaHRtLmhpbmV0Lm5ldDo4MDgwL3VwbG9hZAo='
        speed_test 'HTTP' '乐天移动' '东京' '' 'aHR0cDovL29va2xhLm1ic3BlZWQubmV0OjgwODAvZG93bmxvYWQK' 'aHR0cDovL29va2xhLm1ic3BlZWQubmV0OjgwODAvdXBsb2FkCg=='
        speed_test 'HTTP' 'Kdatacenter ' '首尔' '' 'aHR0cDovL3NwZWVkdGVzdC5rZGF0YWNlbnRlci5jb206ODA4MC9kb3dubG9hZAo=' 'aHR0cDovL3NwZWVkdGVzdC5rZGF0YWNlbnRlci5jb206ODA4MC91cGxvYWQK'
    fi

    end=$(date +%s)
    echo -e "\r——————————————————————————————————————————————————————————————————————"

    if [[ "$thread" == "" ]]; then
        echo -ne "  单线程"
    else
        echo -ne "  多线程"
    fi

    time=$(( $end - $start ))
    if [[ $time -gt 60 ]]; then
        min=$(expr $time / 60)
        sec=$(expr $time % 60)
        echo -e "测试完成, 本次测速耗时: ${min} 分 ${sec} 秒"
    else
        echo -e "测试完成, 本次测速耗时: ${time} 秒"
    fi
    echo -ne "  当前时间: "
    echo $(TZ=Asia/Shanghai date --rfc-3339=seconds)
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
